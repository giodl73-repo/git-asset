//! Exact completed-restoration transition; ordinary producer edits remain immutable.
use super::*;
use crate::annex::publication::{Resources, restore as native};

pub struct Target<'a> {
    pub profile: &'a Profile,
    pub namespace: &'a mut Namespace,
    pub nodes: &'a mut MetadataStore,
}
pub struct Baseline {
    pub version: Version,
    pub record: Object,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    restored: native::Plan,
    update: super::Plan,
}
impl Plan {
    /// Original native restore recipe; this accessor performs no media IO.
    pub fn restoration(&self) -> &native::Plan {
        &self.restored
    }
    fn record(&self) -> Result<Record> {
        Record::new(self.restored.publication().clone(), None)
    }
    fn valid(&self) -> Result<()> {
        self.update.valid()?;
        native::Plan::from_json(&self.restored.to_json()?)?;
        let row = self.record()?;
        if self.schema != "git-asset.producer-restore-update.experimental-1"
            || self.update.schema != RESTORE_SCHEMA
            || self.update.expected.is_none()
            || self.update.records != [(row.object.clone(), row.metadata_object()?)]
        {
            return Err(Error::Integrity);
        }
        Ok(())
    }
    pub fn root(&self) -> &Object {
        self.update.root()
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.valid()?;
        encode(self)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 65536 {
            return Err(Error::Budget);
        }
        let plan: Self = serde_json::from_slice(raw).map_err(|_| Error::Invalid)?;
        if plan.to_json()? != raw {
            return Err(Error::Integrity);
        }
        Ok(plan)
    }
}
fn transition(
    old: Option<&Record>,
    new: &Record,
    original: native::Original<'_>,
    row: &Record,
) -> Result<()> {
    let old = old.ok_or(Error::Conflict)?;
    if old.object != row.object
        || !old.publications().any(|p| p == original.publication)
        || new != row
    {
        return Err(Error::Conflict);
    }
    Ok(())
}
/// Reopen and hold completed bytes while preparing one exact metadata row edit.
pub fn prepare(
    target: &mut Target<'_>,
    baseline: &Baseline,
    context: &native::Context<'_>,
    original: native::Original<'_>,
    restored: &native::Plan,
    resources: &mut Resources<'_>,
) -> Result<Plan> {
    let _bytes = native::reopen(context, original, restored, resources)?;
    let row = Record::new(restored.publication().clone(), None)?;
    let mut update = prepare_checked(
        target.profile,
        Some(&baseline.version),
        &[Change {
            expected_record: Some(baseline.record.clone()),
            record: row.clone(),
        }],
        target.namespace,
        target.nodes,
        |old, new| transition(old, new, original, &row),
    )?
    .ok_or(Error::Conflict)?;
    update.schema = RESTORE_SCHEMA.into();
    let plan = Plan {
        schema: "git-asset.producer-restore-update.experimental-1".into(),
        restored: restored.clone(),
        update,
    };
    plan.to_json()?;
    Ok(plan)
}
/// Historical receipt first; never reselect an older root or inspect media on replay.
pub fn status(target: &mut Target<'_>, plan: &Plan) -> Result<Option<Report>> {
    plan.valid()?;
    super::status(target.profile, &plan.update, target.namespace, target.nodes)
}
/// Persist this exact Plan first. Fresh native guards span atomic root adoption.
pub fn apply(
    target: &mut Target<'_>,
    context: &native::Context<'_>,
    original: native::Original<'_>,
    plan: &Plan,
    resources: &mut Resources<'_>,
    admit: impl FnMut() -> Result<()>,
) -> Result<Report> {
    if let Some(report) = status(target, plan)? {
        return Ok(report);
    }
    let _bytes = native::reopen(context, original, &plan.restored, resources)?;
    let row = plan.record()?;
    apply_checked(
        target.profile,
        &plan.update,
        target.namespace,
        target.nodes,
        admit,
        |old, new| transition(old, new, original, &row),
    )
}
