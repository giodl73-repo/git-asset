//! Admitted single-keeper planning over existing retirement. No native removal.
use super::*;
use crate::annex::control;
use git_asset_store::{
    Key, accounted,
    ledger::CapacityLedger,
    replica::ReplicaEnrollment,
    retention::{self, RetentionGrant},
    retirement as metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorBoundary {
    #[default]
    Unqualified,
    NamespaceOwned,
    MixedLegacy,
}
/// Trusted private owner input, never event-supplied authority. Domain admission
/// covers each native pool and its control store; observed devices alone do not.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub source: control::Profile,
    pub keeper: control::Profile,
    pub source_domain: Key,
    pub keeper_domain: Key,
    #[serde(default)]
    pub allow_evict: bool,
    #[serde(default)]
    pub unshared_direct_backing: bool,
    #[serde(default)]
    pub actor_boundary: ActorBoundary,
}
fn digest(value: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(&encode(value)?))
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let bytes = git_asset_core::identity::canonical_json(
        &serde_json::to_value(value).map_err(|_| publication::Error::Invalid)?,
    )
    .map_err(|_| publication::Error::Invalid)?;
    if bytes.len() > 65536 {
        return Err(publication::Error::Budget.into());
    }
    Ok(bytes)
}
impl Profile {
    /// Pure policy check. This is not verification of a mixed-authority handoff.
    pub fn admit(&self, source: &Digest, keeper: &Digest) -> Result<Digest> {
        if !self.allow_evict || !self.unshared_direct_backing {
            return Err(publication::Error::Denied.into());
        }
        match self.actor_boundary {
            ActorBoundary::NamespaceOwned => (),
            ActorBoundary::Unqualified => return Err(publication::Error::Denied.into()),
            ActorBoundary::MixedLegacy => return Err(publication::Error::Unsupported.into()),
        }
        if source == keeper
            || self.source.store == self.keeper.store
            || self.source_domain == self.keeper_domain
        {
            return Err(publication::Error::Denied.into());
        }
        digest(&(
            "git-asset.native-eviction-owner.experimental-1",
            self.source.binding(source)?,
            self.keeper.binding(keeper)?,
            &self.source_domain,
            &self.keeper_domain,
            self.actor_boundary,
            self.allow_evict,
            self.unshared_direct_backing,
        ))
    }
}
/// Immutable preview. It must be re-admitted and reverified by an executor;
/// decoding this document grants no removal permission or live custody.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    intent: Intent,
    metadata: metadata::Plan,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    object: Object,
    source: Digest,
    keeper: Digest,
    policy: Digest,
    source_producer: Digest,
    keeper_producer: Digest,
    basis: Digest,
    device: Digest,
    keeper_locations: Digest,
    snapshot: snapshot::Snapshot,
}
impl Intent {
    fn context(&self) -> Result<Digest> {
        digest(&(
            "git-asset.native-eviction-context.experimental-1",
            &self.object,
            &self.source,
            &self.keeper,
            &self.policy,
            &self.source_producer,
            &self.keeper_producer,
            &self.basis,
            &self.device,
            &self.keeper_locations,
            &self.snapshot,
        ))
    }
}
impl Plan {
    pub fn metadata_plan(&self) -> &metadata::Plan {
        &self.metadata
    }
    pub fn object(&self) -> &Object {
        &self.intent.object
    }
    pub fn id(&self) -> Result<Digest> {
        Ok(Digest::blob(&self.to_json()?))
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let request = self.metadata.request();
        if self.schema != "git-asset.native-eviction-plan.experimental-1"
            || self.intent.source == self.intent.keeper
            || self.intent.snapshot.object != self.intent.object
            || request.context != self.intent.context()?
            || request.keepers.len() != 1
            || request.required_domains != 1
            || request.publication.object
                != Object::from_bytes(&control::document(&self.intent.object)?)
        {
            return Err(publication::Error::Invalid.into());
        }
        self.metadata
            .id()
            .map_err(|_| publication::Error::Invalid)?;
        encode(self)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 65536 {
            return Err(publication::Error::Budget.into());
        }
        let value =
            git_asset_core::identity::parse_json(raw).map_err(|_| publication::Error::Invalid)?;
        let plan: Self = serde_json::from_value(value).map_err(|_| publication::Error::Invalid)?;
        if plan.to_json()? != raw {
            return Err(publication::Error::Invalid.into());
        }
        Ok(plan)
    }
}
fn retention_error(error: retention::RetentionError) -> Error {
    use retention::RetentionError as E;
    match error {
        E::InvalidPlan => publication::Error::Integrity,
        E::Released | E::Invalidated => publication::Error::Stale,
        E::Store(e) => publication::Error::from(accounted::AccountedError::Store(e)),
        E::Ledger(e) => publication::Error::from(accounted::AccountedError::Ledger(e)),
        _ => publication::Error::NeedsRecovery,
    }
    .into()
}
impl StorageSeparation<'_, '_> {
    /// Explicit readonly plan with exactly one independently retained keeper.
    /// Source/keeper completion tokens keep current physical origins live here.
    pub fn plan(
        &self,
        profile: &Profile,
        operation: Key,
        source_read: &SourceRead,
        source: &publication::ProducerCompletion,
        keeper: &publication::ProducerCompletion,
        grant: RetentionGrant,
    ) -> Result<Plan> {
        let source_store = self.keeper._source.store();
        let keeper_store = self.keeper.store();
        let policy = profile.admit(&source_store, keeper_store)?;
        self.revalidate()?;
        let basis = self.keeper._source.replay_basis()?.id()?;
        if source.pool != source_store
            || &keeper.pool != keeper_store
            || source.object != self.keeper.snapshot.object
            || keeper.object != source.object
            || source.configuration != self.keeper._source.profile.pool.configuration
            || &keeper.configuration != self.keeper._configuration.object()
            || source.control_binding != profile.source.binding(&source_store)?
            || keeper.control_binding != profile.keeper.binding(keeper_store)?
            || source_read.basis != basis
            || source_read.read.object() != &source.object
        {
            return Err(publication::Error::Stale.into());
        }
        control::configuration(
            &self.keeper._source.profile.pool,
            &self
                .keeper
                ._source
                .profile
                .binding
                .id()
                .map_err(protection_error)?,
            Some(&source.control_binding),
        )?;
        control::configuration(
            &self.keeper.profile.pool,
            &self.keeper.profile.binding.id().map_err(protection_error)?,
            Some(&keeper.control_binding),
        )?;
        let source_metadata = profile.source.open_store(&source_store)?;
        let keeper_metadata = profile.keeper.open_store(keeper_store)?;
        let mut keeper_ledger =
            CapacityLedger::open(&profile.keeper.ledger, &profile.keeper.minimum)
                .map_err(|e| publication::Error::from(accounted::AccountedError::Ledger(e)))?;
        if grant.store() != keeper_metadata.identity()
            || grant.request().publication.object != source.control.publication.request().object
        {
            return Err(publication::Error::Denied.into());
        }
        let mut retained = retention::verify(&mut keeper_ledger, &keeper_metadata, &grant)
            .map_err(retention_error)?;
        let intent = Intent {
            object: source.object.clone(),
            source: source_store.clone(),
            keeper: keeper_store.clone(),
            policy,
            source_producer: source.plan.clone(),
            keeper_producer: keeper.plan.clone(),
            basis,
            device: self.id()?,
            keeper_locations: digest(&(
                &self.keeper.profile.pool.repository,
                self.keeper
                    .profile
                    .members
                    .iter()
                    .map(|m| (&m.root, m.minimum.identity()))
                    .collect::<Vec<_>>(),
            ))?,
            snapshot: self.keeper.snapshot.clone(),
        };
        let metadata = metadata::plan(
            &source_metadata,
            metadata::Request {
                operation,
                owner: profile.source.owner.clone(),
                publication: source.control.publication.request().clone(),
                context: intent.context()?,
                source: ReplicaEnrollment {
                    store: profile.source.store.clone(),
                    failure_domain: profile.source_domain.clone(),
                },
                keepers: vec![metadata::Keeper {
                    grant,
                    enrollment: ReplicaEnrollment {
                        store: profile.keeper.store.clone(),
                        failure_domain: profile.keeper_domain.clone(),
                    },
                }],
                required_domains: 1,
                capacity: profile.source.minimum.identity().clone(),
            },
        )
        .map_err(|_| publication::Error::Stale)?;
        let plan = Plan {
            schema: "git-asset.native-eviction-plan.experimental-1".into(),
            intent,
            metadata,
        };
        retained
            .revalidate(&mut keeper_ledger, &keeper_metadata)
            .map_err(retention_error)?;
        self.revalidate()?;
        plan.id()?;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_policy_defaults_deny_without_opening_or_initializing_roots() {
        let root = tempfile::tempdir().unwrap();
        let control = |name: &str| {
            serde_json::json!({
                "root":root.path().join(name), "store":Digest::blob(name.as_bytes()),
                "ledger":root.path().join(format!("{name}-ledger")),
                "minimum":{"genesis":Digest::blob(name.as_bytes()),"sequence":0,"hash":Digest::blob(b"header")},
                "owner":"controller"
            })
        };
        let mut profile: Profile = serde_json::from_value(serde_json::json!({
            "source":control("source"),"keeper":control("keeper"),
            "source_domain":"one","keeper_domain":"two"
        }))
        .unwrap();
        let source = Digest::blob(b"native-source");
        let keeper = Digest::blob(b"native-keeper");
        assert_eq!(
            profile.admit(&source, &keeper).unwrap_err().code(),
            "denied"
        );
        profile.allow_evict = true;
        profile.unshared_direct_backing = true;
        assert_eq!(
            profile.admit(&source, &keeper).unwrap_err().code(),
            "denied"
        );
        profile.actor_boundary = ActorBoundary::MixedLegacy;
        assert_eq!(
            profile.admit(&source, &keeper).unwrap_err().code(),
            "unsupported"
        );
        profile.actor_boundary = ActorBoundary::NamespaceOwned;
        let original = profile.admit(&source, &keeper).unwrap();
        profile.keeper.root = root.path().join("a-clone-is-another-binding");
        assert_ne!(original, profile.admit(&source, &keeper).unwrap());
        profile.keeper_domain = profile.source_domain.clone();
        assert_eq!(
            profile.admit(&source, &keeper).unwrap_err().code(),
            "denied"
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
