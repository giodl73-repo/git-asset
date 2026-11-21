//! Native operation metadata adapter over the existing accounted/lifecycle engine.
//! The control document and its read guard never prove native byte custody.
use super::{
    process,
    publication::{Error, Result},
};
use git_asset_core::{Digest, Object, identity};
use git_asset_store::{
    AckLevel, Key, Request, accounted,
    ledger::{CapacityLedger, Checkpoint, JournalRef},
    local::LocalStore,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub root: PathBuf,
    pub store: Digest,
    pub ledger: PathBuf,
    pub minimum: Checkpoint,
    pub owner: Key,
    #[serde(default)]
    pub allow_reacquire: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub publication: accounted::PublicationPlan,
    pub job: JournalRef,
}
pub(super) struct Use {
    _published: accounted::Published,
}
impl Use {
    pub(super) fn allows_missing(
        &self,
        profile: &super::publication::Profile,
        control: &Profile,
        request: &super::publication::Request,
        plan: &Plan,
    ) -> Result<bool> {
        let origin = self._published.origin().ok_or(Error::NeedsRecovery)?;
        let context = writer_context(profile, request)?;
        let expected = if origin.reacquisition().is_some() {
            if !control.allow_reacquire {
                return Err(Error::Denied);
            }
            digest(&("reacquire", origin, &context))?
        } else {
            context
        };
        Ok(plan.publication.context() == Some(&expected))
    }
}
fn digest(v: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(
        &identity::canonical_json(&serde_json::to_value(v).map_err(|_| Error::Invalid)?)
            .map_err(|_| Error::Invalid)?,
    ))
}
pub fn document(object: &Object) -> Result<Vec<u8>> {
    identity::canonical_json(&serde_json::json!({"schema":"git-asset.annex-operation-control.experimental-1","object":object}))
        .map_err(|_| Error::Invalid)
}
pub fn place(native_store: &Digest) -> Result<Key> {
    Key::parse(&format!("native-control-{}", native_store.as_str())).map_err(|_| Error::Invalid)
}
pub fn initial_request(object: &Object, owner: Key) -> Result<Request> {
    let object = Object::from_bytes(&document(object)?);
    Ok(Request {
        operation: Key::parse(&format!("control-{}", object.sha256.as_str()))
            .map_err(|_| Error::Invalid)?,
        owner,
        object,
        required: AckLevel::ProcessRestart,
    })
}
/// Bind the original native producer, not another alias or a moving/latest selection.
pub fn writer_context(
    profile: &super::publication::Profile,
    request: &super::publication::Request,
) -> Result<Digest> {
    let native = super::Reader::new(profile.pool.pool.clone())?.store();
    digest(&(
        "git-asset.annex-control-producer.experimental-1",
        native,
        &profile.pool.pool.configuration,
        profile.pool.binding.id().map_err(|_| Error::Invalid)?,
        request,
    ))
}
impl Profile {
    pub fn binding(&self, native: &Digest) -> Result<Digest> {
        if !super::absolute(&self.root) || !super::absolute(&self.ledger) {
            return Err(Error::Invalid);
        }
        digest(&(
            "git-asset.annex-control-binding.experimental-1",
            native,
            &self.root,
            &self.store,
            &self.ledger,
            self.minimum.identity(),
            &self.owner,
            self.allow_reacquire,
        ))
    }
    pub(super) fn check(&self, native: &Digest, object: &Object, plan: &Plan) -> Result<()> {
        self.binding(native)?;
        let request = plan.publication.request();
        if request.owner != self.owner
            || request.required != AckLevel::ProcessRestart
            || request.object != Object::from_bytes(&document(object)?)
            || plan.publication.context().is_none()
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    pub(super) fn open(&self, native: &Digest, object: &Object, plan: &Plan) -> Result<Use> {
        self.check(native, object, plan)?;
        let store = LocalStore::open(&self.root).map_err(store_error)?;
        if store.identity() != &self.store
            || store.place() != &place(native)?
            || !store.supports_retirement()
            || store.final_policy().is_some()
        {
            return Err(Error::Stale);
        }
        let mut ledger = CapacityLedger::open(&self.ledger, &self.minimum)
            .map_err(|e| Error::from(accounted::AccountedError::Ledger(e)))?;
        let published = accounted::reopen(&mut ledger, &store, &plan.job, &plan.publication)
            .map_err(Error::from)?;
        let origin = published.origin().ok_or(Error::NeedsRecovery)?;
        let expected = match origin.reacquisition() {
            Some(request) => request.clone(),
            None => initial_request(object, self.owner.clone())?,
        };
        if published.publication().request() != &expected {
            return Err(Error::Conflict);
        }
        Ok(Use {
            _published: published,
        })
    }
}
fn store_error(e: git_asset_store::Error) -> Error {
    use git_asset_store::Error as E;
    match e {
        E::OperationConflict => Error::Conflict,
        E::AuthorizedAbsence | E::ReacquisitionRequired => Error::Stale,
        E::Busy => Error::Busy,
        E::Budget => Error::Budget,
        E::Integrity | E::CorruptRecord => Error::Integrity,
        _ => Error::NeedsRecovery,
    }
}
impl From<accounted::AccountedError> for Error {
    fn from(e: accounted::AccountedError) -> Self {
        use accounted::AccountedError as E;
        match e {
            E::Store(e) => store_error(e),
            E::Ledger(git_asset_store::ledger::LedgerError::Busy) => Error::Busy,
            E::Ledger(git_asset_store::ledger::LedgerError::Conflict) => Error::Conflict,
            E::Ledger(git_asset_store::ledger::LedgerError::Budget) => Error::Budget,
            E::InvalidPlan => Error::Invalid,
            _ => Error::NeedsRecovery,
        }
    }
}
pub(super) fn configuration(
    pool: &super::Profile,
    binding: &Digest,
    expected: Option<&Digest>,
) -> Result<()> {
    let pattern = format!(
        "^({}|gitasset[.]lifecyclecontrol)$",
        super::managed::PROTECTION_CONFIG.replace('.', "[.]")
    );
    let output = process::run(
        pool,
        false,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get-regexp",
            &pattern,
        ],
    )?;
    let text = std::str::from_utf8(&output.stdout).map_err(|_| Error::Invalid)?;
    let (mut protection, mut control) = (None, None);
    for line in text.lines() {
        let (key, value) = line.split_once(' ').ok_or(Error::Stale)?;
        let slot = match key {
            super::managed::PROTECTION_CONFIG => &mut protection,
            "gitasset.lifecyclecontrol" => &mut control,
            _ => return Err(Error::Stale),
        };
        if slot.replace(value).is_some() {
            return Err(Error::Stale);
        }
    }
    if protection != Some(binding.as_str()) || control != expected.map(Digest::as_str) {
        return Err(Error::Stale);
    }
    Ok(())
}
