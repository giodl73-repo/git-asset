//! Shared immutable incarnation records. Pure record construction is not admission,
//! an observed birth, terminal evidence, custody or authority to perform IO.
use crate::{AckLevel, Error, Request, Result};
use git_asset_core::{Digest, Object, identity};
use serde::{Deserialize, Serialize};

pub const MAX_BIRTHS: usize = 16;
// Keep the existing wire identifier at its frozen compatibility location.
use crate::local::lifecycle::SCHEMA;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Incarnation {
    pub(crate) schema: String,
    pub(crate) store: Digest,
    pub(crate) object: Object,
    pub(crate) index: usize,
    pub(crate) previous: Option<Digest>,
    pub(crate) reacquire: Option<Request>,
    pub(crate) placement: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) final_context: Option<Digest>,
}
pub(crate) fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let bytes =
        identity::canonical_json(&serde_json::to_value(value).map_err(|_| Error::CorruptRecord)?)
            .map_err(|_| Error::CorruptRecord)?;
    if bytes.len() > 65536 {
        return Err(Error::Budget);
    }
    Ok(bytes)
}
pub(crate) fn digest(value: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(&encode(value)?))
}
impl Incarnation {
    pub fn initial(store: Digest, object: Object) -> Result<Self> {
        Ok(Self {
            schema: SCHEMA.into(),
            placement: digest(&("initial-placement", &store, &object))?,
            store,
            object,
            index: 0,
            previous: None,
            reacquire: None,
            final_context: None,
        })
    }
    /// Caller must validate the actual terminal and predecessor chain under its fence.
    pub fn cancelled(&self, terminal: Digest) -> Result<Self> {
        self.next(terminal, None, None)
    }
    pub fn reacquired(&self, terminal: Digest, request: Request) -> Result<Self> {
        self.next(terminal, Some(request), None)
    }
    pub fn final_reacquired(
        &self,
        terminal: Digest,
        request: Request,
        context: Digest,
    ) -> Result<Self> {
        self.next(terminal, Some(request), Some(context))
    }
    fn next(
        &self,
        terminal: Digest,
        request: Option<Request>,
        context: Option<Digest>,
    ) -> Result<Self> {
        let index = self.index.checked_add(1).ok_or(Error::Budget)?;
        if index >= MAX_BIRTHS {
            return Err(Error::Budget);
        }
        if self.schema != SCHEMA
            || request
                .as_ref()
                .is_some_and(|r| r.object != self.object || r.required > AckLevel::ProcessRestart)
        {
            return Err(Error::InvalidRequest);
        }
        let placement = match (&request, &context) {
            (Some(r), Some(c)) => digest(&(
                "final-reacquired-placement",
                &self.store,
                &self.object,
                &terminal,
                r,
                c,
            ))?,
            (Some(r), None) => digest(&(
                "reacquired-placement",
                &self.store,
                &self.object,
                &terminal,
                r,
            ))?,
            (None, None) => self.placement.clone(),
            (None, Some(_)) => return Err(Error::InvalidRequest),
        };
        Ok(Self {
            schema: SCHEMA.into(),
            store: self.store.clone(),
            object: self.object.clone(),
            index,
            previous: Some(terminal),
            reacquire: request,
            placement,
            final_context: context,
        })
    }
    pub fn store(&self) -> &Digest {
        &self.store
    }
    pub fn object(&self) -> &Object {
        &self.object
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn placement(&self) -> &Digest {
        &self.placement
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        encode(self)
    }
    pub fn id(&self) -> Result<Digest> {
        digest(self)
    }
}
#[cfg(test)]
mod tests;
