//! Explicitly enrolled index heads, frozen for one authenticated pass.
use super::*;
use crate::annex::producer_index::update::{HeldRoot, Profile};
use git_asset_core::identity;
use git_asset_store::{
    metadata::MetadataReader,
    source_claims::{Checkpoint, Version},
};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub root: PathBuf,
    pub minimum: Checkpoint,
    pub profile: Profile,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heads {
    pub source: Head,
    pub destination: Head,
}
#[derive(Clone, Serialize)]
pub struct Pin {
    pub namespace: Digest,
    pub source: Key,
    pub revision: u64,
    pub version: Version,
}
#[derive(Clone, Serialize)]
pub struct Pins {
    pub source: Pin,
    pub destination: Pin,
}
pub(super) struct Resolved {
    pub roots: ProducerRoots,
    pub pins: Option<Pins>,
    _uses: Vec<HeldRoot>,
}
impl Resolved {
    pub fn basis(&self, original: Digest) -> Result<Digest, Error> {
        let Some(pins) = &self.pins else {
            return Ok(original);
        };
        Ok(Digest::blob(
            &identity::canonical_json(&serde_json::json!([
                "git-asset.native-producer-heads-basis.experimental-1",
                original,
                pins
            ]))
            .map_err(|_| publication::Error::Invalid)?,
        ))
    }
}
impl Head {
    pub(super) fn hold(&self, nodes: &MetadataReader) -> Result<(Pin, HeldRoot), Error> {
        let mut namespace =
            Namespace::open(&self.root, &self.minimum).map_err(publication::Error::from)?;
        let held = self.profile.hold_current(&mut namespace, nodes)?;
        let pin = Pin {
            namespace: namespace.identity().clone(),
            source: self.profile.scope.prefix.clone(),
            revision: held.revision(),
            version: held.version().clone(),
        };
        Ok((pin, held))
    }
}
impl Heads {
    pub(super) fn validate(&self) -> Result<(), Error> {
        for head in [&self.source, &self.destination] {
            head.profile.scope.id()?;
            head.profile
                .limits
                .validate()
                .map_err(|_| publication::Error::Budget)?;
            if head.minimum.identity() != &head.profile.scope.namespace {
                return Err(Error::Admission(AdmissionError::Trust));
            }
        }
        if self.source.profile.scope.namespace == self.destination.profile.scope.namespace
            && self.source.profile.scope.prefix == self.destination.profile.scope.prefix
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        Ok(())
    }
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub(super) fn resolve_producers(
        &self,
        source: &MetadataReader,
        destination: &MetadataReader,
    ) -> Result<Resolved, Error> {
        if let Some(roots) = &self.policy.producers {
            if self.policy.producer_heads.is_some() {
                return Err(Error::Admission(AdmissionError::Trust));
            }
            return Ok(Resolved {
                roots: roots.clone(),
                pins: None,
                _uses: Vec::new(),
            });
        }
        let heads = self
            .policy
            .producer_heads
            .as_ref()
            .ok_or(publication::Error::Unsupported)?;
        heads.validate()?;
        let (a, held_a) = heads.source.hold(source)?;
        let (b, held_b) = heads.destination.hold(destination)?;
        Ok(Resolved {
            roots: ProducerRoots {
                source: a.version.selection.root.clone(),
                destination: b.version.selection.root.clone(),
            },
            pins: Some(Pins {
                source: a,
                destination: b,
            }),
            _uses: vec![held_a, held_b],
        })
    }
}
