//! Owner registration of an already published, protected native producer.
use super::*;
use crate::annex::publication::{Plan as Producer, Publisher, Resources};

pub struct Target<'a> {
    pub profile: &'a Profile,
    pub namespace: &'a mut Namespace,
    pub nodes: &'a mut MetadataStore,
}

/// Prepare an absent-row insert. Persist the exact plan and producer before apply.
pub fn prepare(
    target: &mut Target<'_>,
    expected: Option<&Version>,
    producer: &Producer,
    publisher: &mut Publisher,
    resources: &mut Resources<'_>,
) -> Result<Option<Plan>> {
    let _bytes = publisher.reopen(producer, resources)?;
    super::prepare(
        target.profile,
        expected,
        &[Change {
            expected_record: None,
            record: Record::new(producer.clone(), None)?,
        }],
        target.namespace,
        target.nodes,
    )
}

/// Recover the original receipt or reopen the protected bytes through the CAS.
pub fn apply(
    target: &mut Target<'_>,
    plan: &Plan,
    producer: &Producer,
    publisher: &mut Publisher,
    resources: &mut Resources<'_>,
    admit: impl FnMut() -> Result<()>,
) -> Result<Report> {
    let record = Record::new(producer.clone(), None)?;
    plan.check_record(&record)?;
    if let Some(report) = super::status(target.profile, plan, target.namespace, target.nodes)? {
        return Ok(report);
    }
    if let Some(expected) = plan.expected() {
        let (existing, _) = Index::from_root(expected.selection.root.clone()).get(
            &record.object,
            target.nodes,
            limits(target.profile),
        )?;
        if existing.is_some() {
            return Err(Error::Conflict);
        }
    }
    let _bytes = publisher.reopen(producer, resources)?;
    super::apply(target.profile, plan, target.namespace, target.nodes, admit)
}
