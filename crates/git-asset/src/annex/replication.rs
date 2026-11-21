//! Retained native copy by composition, never an eviction or creative-selection grant.
use super::{managed, publication};
use git_asset_core::{Digest, identity};
use git_asset_store::Key;
pub use publication::{Error, Result, Stage};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};

#[derive(Clone)]
pub struct Profile {
    pub source: publication::Profile,
    pub destination: publication::Profile,
    /// Admitted failure domains; UUIDs and drive aliases do not establish independence.
    pub source_domain: Key,
    pub destination_domain: Key,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    source: publication::Plan,
    destination: publication::Plan,
    source_domain: Key,
    destination_domain: Key,
    locations: Digest,
}
fn digest(value: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(
        &identity::canonical_json(&serde_json::to_value(value).map_err(|_| Error::Invalid)?)
            .map_err(|_| Error::Invalid)?,
    ))
}
impl Plan {
    /// Pure plan admission. Source must already have an exact retained publication.
    pub fn new(
        profile: &Profile,
        source: publication::Plan,
        destination: publication::Plan,
    ) -> Result<Self> {
        source.check_profile(&profile.source)?;
        destination.check_profile(&profile.destination)?;
        if source.candidate() != destination.candidate() {
            return Err(Error::Conflict);
        }
        let reader = managed::Reader::new(profile.source.pool.clone())?;
        let target = managed::Reader::new(profile.destination.pool.clone())?;
        if reader.store() == target.store() {
            return Err(Error::Invalid);
        }
        let limits = profile.source.pool.pool.limits;
        let bytes = source.candidate().object.bytes;
        if limits.live_readers < 2
            || limits.requests < 2
            || limits.object_bytes < bytes
            || limits.total_bytes < bytes.checked_mul(2).ok_or(Error::Budget)?
        {
            return Err(Error::Budget);
        }
        let locations = digest(&(
            &profile.source.pool.pool.repository,
            profile
                .source
                .pool
                .members
                .iter()
                .map(|m| (&m.root, &m.minimum))
                .collect::<Vec<_>>(),
            &profile.destination.pool.pool.repository,
            profile
                .destination
                .pool
                .members
                .iter()
                .map(|m| (&m.root, &m.minimum))
                .collect::<Vec<_>>(),
        ))?;
        let context = digest(&(
            "git-asset.annex-copy.experimental-1",
            source.id()?,
            &locations,
            &profile.source_domain,
            &profile.destination_domain,
        ))?;
        Ok(Self {
            source,
            destination: destination.with_context(Some(context)),
            source_domain: profile.source_domain.clone(),
            destination_domain: profile.destination_domain.clone(),
            locations,
        })
    }
    pub fn id(&self) -> Result<Digest> {
        digest(self)
    }
    pub fn source(&self) -> &publication::Plan {
        &self.source
    }
    pub fn destination(&self) -> &publication::Plan {
        &self.destination
    }
    pub fn required_peak(&self, witness: &git_asset_store::local::LocalStore) -> Result<u64> {
        self.destination.required_peak(witness)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub plan: Digest,
    pub destination: publication::Receipt,
    /// Historical report only, not a fresh custody assessment or deletion grant.
    pub source_verified: bool,
    pub declared_domains: usize,
}
pub struct Ready {
    receipt: Receipt,
    destination: publication::Ready,
    _source: Option<publication::Ready>,
}
impl Ready {
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
    pub fn recovery(&self) -> &publication::Recovery {
        self.destination.recovery()
    }
    pub fn source_recovery(&self) -> Option<&publication::Recovery> {
        self._source.as_ref().map(publication::Ready::recovery)
    }
    pub fn destination(&self) -> &publication::Ready {
        &self.destination
    }
}
impl Read for Ready {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        self.destination.read(b)
    }
}
impl Seek for Ready {
    fn seek(&mut self, p: std::io::SeekFrom) -> std::io::Result<u64> {
        self.destination.seek(p)
    }
}
pub struct Copier {
    profile: Profile,
    source: publication::Publisher,
    input: managed::Reader,
    destination: publication::Publisher,
}
impl Copier {
    /// Pure; no repository initialization, byte lookup or source census.
    pub fn new(profile: Profile) -> Result<Self> {
        Ok(Self {
            source: publication::Publisher::new(profile.source.clone())?,
            input: managed::Reader::new(profile.source.pool.clone())?,
            destination: publication::Publisher::new(profile.destination.clone())?,
            profile,
        })
    }
    fn check(&self, plan: &Plan) -> Result<()> {
        if &Plan::new(&self.profile, plan.source.clone(), plan.destination.clone())? != plan {
            return Err(Error::Stale);
        }
        Ok(())
    }
    pub fn status(
        &mut self,
        plan: &Plan,
        resources: &mut publication::Resources<'_>,
    ) -> Result<publication::Status> {
        self.check(plan)?;
        self.destination.status(&plan.destination, resources)
    }
    fn ready(
        plan: &Plan,
        destination: publication::Ready,
        source: Option<publication::Ready>,
    ) -> Result<Ready> {
        Ok(Ready {
            receipt: Receipt {
                plan: plan.id()?,
                destination: destination.receipt().clone(),
                source_verified: source.is_some(),
                declared_domains: if source.is_some()
                    && plan.source_domain != plan.destination_domain
                {
                    2
                } else {
                    1
                },
            },
            destination,
            _source: source,
        })
    }
    pub fn apply(
        &mut self,
        plan: &Plan,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
    ) -> Result<Ready> {
        self.apply_hook(plan, source, destination, |_| Ok(()))
    }
    pub fn apply_hook(
        &mut self,
        plan: &Plan,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        hook: impl FnMut(Stage) -> Result<()>,
    ) -> Result<Ready> {
        self.check(plan)?;
        // Preserve native publication's default-denied no-IO boundary.
        if !self.profile.destination.allow_publish {
            return Err(Error::Denied);
        }
        let held = self.source.reopen(&plan.source, source)?;
        let copied =
            self.destination
                .apply_hook(&plan.destination, destination, &mut self.input, hook)?;
        Self::ready(plan, copied, Some(held))
    }
    /// Recover the original destination; incomplete stages require apply with source.
    pub fn resume(
        &mut self,
        plan: &Plan,
        destination: &mut publication::Resources<'_>,
    ) -> Result<Ready> {
        self.check(plan)?;
        let ready = self.destination.resume(&plan.destination, destination)?;
        Self::ready(plan, ready, None)
    }
    /// Fresh retained destination readback; never accesses the source pool.
    pub fn reopen(
        &mut self,
        plan: &Plan,
        destination: &mut publication::Resources<'_>,
    ) -> Result<Ready> {
        self.check(plan)?;
        let ready = self.destination.reopen(&plan.destination, destination)?;
        Self::ready(plan, ready, None)
    }
}
