//! Shared explicit retained read; original ingest protection is not substituted.
use super::*;
use crate::annex::snapshot::{Held, Snapshot};
use git_asset_store::source_claims::{Checkpoint, admission::source_use};
/// Actual byte/config/control/namespace guards and exact keeper generation.
/// No clone, serialization or expiry; dropping never removes durable claims.
pub struct Retained {
    observed: Observed,
    snapshot: Held,
    _generation: source_use::Use,
    recovery: Recovery,
    minimum: Checkpoint,
}
impl Retained {
    pub fn snapshot(&self) -> &Snapshot {
        self.snapshot.snapshot()
    }
    pub fn recovery(&self) -> &Recovery {
        &self.recovery
    }
    pub fn snapshot_minimum(&self) -> &Checkpoint {
        &self.minimum
    }
}
impl Read for Retained {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.observed
            .read
            .as_mut()
            .ok_or_else(|| std::io::Error::other("retained read unavailable"))?
            .read(bytes)
    }
}
impl Seek for Retained {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        self.observed
            .read
            .as_mut()
            .ok_or_else(|| std::io::Error::other("retained read unavailable"))?
            .seek(position)
    }
}
impl Publisher {
    /// Explicit exact retained member, original publication completion and fresh
    /// bytes. The original per-object ingest claim need not remain selected.
    pub fn open_retained(
        &mut self,
        plan: &Plan,
        snapshot: &Snapshot,
        resources: &mut Resources<'_>,
    ) -> Result<Retained> {
        if snapshot.object != plan.candidate().object
            || !self
                .profile
                .pool
                .binding
                .members()
                .iter()
                .any(|m| m.namespace == snapshot.namespace && m.places.contains(&snapshot.place))
        {
            return Err(Error::Denied);
        }
        // Bind the generation guard to the real admitted roster location.
        let member = self
            .profile
            .pool
            .members
            .iter()
            .find(|m| m.minimum.identity() == &snapshot.namespace)
            .ok_or(Error::Denied)?;
        let mut namespace = Namespace::open(&member.root, &member.minimum)?;
        let state = namespace
            .source(&snapshot.source)?
            .ok_or(Error::ProtectionLost)?;
        if state.owner != snapshot.owner || state.kind != Some(Kind::Retention) {
            return Err(Error::Denied);
        }
        let current = state.current.as_ref().ok_or(Error::Unsupported)?;
        let generation = namespace
            .source_use(&snapshot.source, &snapshot.owner, state.revision, current)
            .map_err(|e| match e {
                source_use::UseError::Unsupported => Error::Unsupported,
                source_use::UseError::Owner => Error::Denied,
                source_use::UseError::Stale | source_use::UseError::Inactive => Error::Stale,
                source_use::UseError::Admission(e) => e.into(),
            })?;
        let held = snapshot.verify(
            &mut namespace,
            resources.metadata,
            TreeLimits {
                max_entries: resources.limits.bindings,
                max_reads: resources.limits.reads,
                max_read_bytes: resources.limits.read_bytes,
                ..TreeLimits::default()
            },
        )?;
        let mut limits = resources.limits.clone();
        limits.reads = limits
            .reads
            .checked_sub(held.metrics().reads)
            .ok_or(Error::Budget)?;
        limits.read_bytes = limits
            .read_bytes
            .checked_sub(held.metrics().read_bytes)
            .ok_or(Error::Budget)?;
        let observed = self.observe(
            plan,
            &mut Resources {
                namespace: &mut *resources.namespace,
                metadata: &mut *resources.metadata,
                ledger: &mut *resources.ledger,
                witness: resources.witness,
                limits,
            },
        )?;
        if !observed.status.registered
            || !observed.status.historical_completion
            || !observed.status.accounting_settled
            || !observed.status.bytes_verified
        {
            return Err(Error::NeedsRecovery);
        }
        Ok(Retained {
            observed,
            snapshot: held,
            _generation: generation,
            recovery: Recovery {
                metadata_minimum: resources.metadata.checkpoint().clone(),
                namespace_minimum: resources.namespace.checkpoint(),
                capacity_minimum: resources.ledger.checkpoint(),
            },
            minimum: namespace.checkpoint(),
        })
    }
}
