//! Bounded ordered passes. Every effect remains admitted by the original backend.
use super::*;
use git_asset_store::placement::Cursor;
use serde_json::Value;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Restore,
    Copy,
    RegisterWorking,
    RegisterRetained,
    History,
    Eviction,
}
const ORDER: [Stage; 6] = [
    Stage::Restore,
    Stage::Copy,
    Stage::RegisterWorking,
    Stage::RegisterRetained,
    Stage::History,
    Stage::Eviction,
];
impl Stage {
    fn schema(self) -> &'static str {
        match self {
            Self::Restore => "git-asset.native-restore-pass-report.experimental-1",
            Self::Copy => "git-asset.native-copy-pass-report.experimental-1",
            Self::RegisterWorking | Self::RegisterRetained => {
                "git-asset.native-registration-report.experimental-1"
            }
            Self::History => "git-asset.native-history-pass-report.experimental-1",
            Self::Eviction => "git-asset.native-eviction-pass-report.experimental-1",
        }
    }
    fn payload(self) -> bool {
        matches!(self, Self::Restore | Self::Copy | Self::Eviction)
    }
    fn accepts(self, state: &str) -> bool {
        match self {
            Self::Restore => matches!(state, "not_applicable" | "not_required" | "indexed"),
            Self::Copy => matches!(state, "present" | "copied" | "backed_up"),
            Self::RegisterWorking | Self::RegisterRetained => state == "recorded",
            Self::History => state == "released",
            Self::Eviction => matches!(state, "required" | "protected" | "evicted" | "reconciled"),
        }
    }
}
/// Expected authenticated receipt frame, not effect authority.
pub struct Binding {
    pub policy: Digest,
    pub provenance: crate::github::Provenance,
}
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Hold {
    Budget,
    Refused(AdmissionError),
    StageHeld,
    InvalidReport,
    NoProgress,
}
#[derive(Serialize)]
pub struct Summary {
    pub stage: Stage,
    pub passes: u32,
    pub object_visits: usize,
    pub prospective_bytes: u64,
    pub complete: bool,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub policy: Digest,
    pub provenance: crate::github::Provenance,
    pub stages: Vec<Summary>,
    pub pages: u32,
    pub object_visits: usize,
    pub prospective_bytes: u64,
    pub current_stage: Stage,
    pub last_stage: Option<Stage>,
    pub last_report: Option<Value>,
    pub hold: Option<Hold>,
    pub ordered_passes_complete: bool,
    pub storage_converged: Option<bool>,
}
/// Fail before resource access; the coordinator does not grant any stage permission.
pub fn validate(policy: &Policy) -> Result<(), AdmissionError> {
    let a = policy.actions.as_ref().ok_or(AdmissionError::Disabled)?;
    let heads = policy
        .producer_heads
        .as_ref()
        .ok_or(AdmissionError::Disabled)?;
    if !a.orchestration
        || !policy.allow_selection
        || !policy.allow_copy
        || !a.backup
        || !a.handoff
        || !a.registration
        || a.pending.is_none()
        || !a
            .history
            .as_ref()
            .is_some_and(|h| h.allow_publish && h.allow_release)
        || !a.eviction.as_ref().is_some_and(|e| e.allow_evict)
        || !heads.source.profile.scope.allow_store
        || !heads.destination.profile.scope.allow_store
        || !policy
            .destination_control
            .as_ref()
            .is_some_and(|c| c.allow_reacquire)
    {
        return Err(AdmissionError::Disabled);
    }
    Ok(())
}
#[derive(Deserialize)]
struct SelectionFrame {
    schema: String,
    #[serde(rename = "plan")]
    _plan: Digest,
    desired_current: bool,
    source_revision: u64,
    observed_ref_current: Option<bool>,
}
struct Page {
    count: usize,
    bytes: u64,
    next: Option<Cursor>,
    fresh: bool,
    held: bool,
    basis: Digest,
}
fn held(value: &Value) -> bool {
    !value.is_null()
}
fn inspect(stage: Stage, value: &Value, binding: &Binding) -> Result<Page, AdmissionError> {
    if value["schema"] != stage.schema()
        || value["policy"]
            != serde_json::to_value(&binding.policy).map_err(|_| AdmissionError::Invalid)?
        || value["provenance"]
            != serde_json::to_value(&binding.provenance).map_err(|_| AdmissionError::Invalid)?
    {
        return Err(AdmissionError::Invalid);
    }
    let selection: SelectionFrame =
        serde_json::from_value(value["selection"].clone()).map_err(|_| AdmissionError::Invalid)?;
    if selection.schema != "tig.git-lifecycle-status.design-1" || selection.source_revision == 0 {
        return Err(AdmissionError::Invalid);
    }
    let outcomes = value["outcomes"]
        .as_array()
        .ok_or(AdmissionError::Invalid)?;
    let mut blocked = !selection.desired_current
        || selection.observed_ref_current != Some(true)
        || held(&value["post_effect_hold"]);
    for out in outcomes {
        blocked |= held(&out["hold"]) || !out["state"].as_str().is_some_and(|s| stage.accepts(s));
        for kind in ["handoff", "eviction"] {
            blocked |=
                held(&out[kind]["failure"]) || held(&out[kind]["mutation"]["post_effect_hold"]);
        }
        for kind in ["native", "registration"] {
            blocked |= held(&out[kind]["post_effect_hold"]);
        }
    }
    if matches!(stage, Stage::RegisterWorking | Stage::RegisterRetained)
        && value["side"]
            != match stage {
                Stage::RegisterWorking => "working",
                _ => "retained",
            }
    {
        return Err(AdmissionError::Invalid);
    }
    let next: Option<Cursor> =
        serde_json::from_value(value["next"].clone()).map_err(|_| AdmissionError::Invalid)?;
    let fresh = match value.get("fresh_pass_required") {
        Some(v) => v.as_bool().ok_or(AdmissionError::Invalid)?,
        None if matches!(
            stage,
            Stage::Restore | Stage::RegisterWorking | Stage::RegisterRetained
        ) =>
        {
            return Err(AdmissionError::Invalid);
        }
        None => false,
    };
    if fresh && next.is_some() {
        return Err(AdmissionError::Invalid);
    }
    let basis =
        serde_json::from_value(value["basis"].clone()).map_err(|_| AdmissionError::Invalid)?;
    Ok(Page {
        count: outcomes.len(),
        bytes: value
            .get("prospective_bytes")
            .map(|v| v.as_u64().ok_or(AdmissionError::Invalid))
            .transpose()?
            .unwrap_or(0),
        next,
        fresh,
        held: blocked,
        basis,
    })
}
/// Start from Restore every time; no caller-supplied stage continuation is proof.
/// Last native report is preserved on holds; originals/older receipts stay asset-side.
pub fn run(
    binding: &Binding,
    budget: &crate::github::placement::Budget,
    mut step: impl FnMut(
        Stage,
        Option<&Cursor>,
        &crate::github::placement::Budget,
    ) -> Result<Value, AdmissionError>,
) -> Result<Report, AdmissionError> {
    budget.validate()?;
    let start = Instant::now();
    let mut report = Report {
        schema: "git-asset.native-ordered-placement.experimental-1",
        policy: binding.policy.clone(),
        provenance: binding.provenance.clone(),
        stages: Vec::new(),
        pages: 0,
        object_visits: 0,
        prospective_bytes: 0,
        current_stage: Stage::Restore,
        last_stage: None,
        last_report: None,
        hold: None,
        ordered_passes_complete: false,
        storage_converged: None,
    };
    for stage in ORDER {
        report.current_stage = stage;
        report.stages.push(Summary {
            stage,
            passes: 0,
            object_visits: 0,
            prospective_bytes: 0,
            complete: false,
        });
        let mut cursor: Option<Cursor> = None;
        let mut fresh_basis = None;
        loop {
            let elapsed = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let remaining_bytes = budget.stream_bytes.saturating_sub(report.prospective_bytes);
            if report.pages >= budget.pages
                || report.object_visits >= budget.objects
                || elapsed >= budget.millis
                || (stage.payload() && remaining_bytes == 0)
            {
                report.hold = Some(Hold::Budget);
                return Ok(report);
            }
            let slice = crate::github::placement::Budget {
                pages: 1,
                objects: (budget.objects - report.object_visits).min(128),
                stream_bytes: remaining_bytes.max(1),
                millis: budget.millis - elapsed,
            };
            report.pages += 1;
            report.stages.last_mut().unwrap().passes += 1;
            let value = match step(stage, cursor.as_ref(), &slice) {
                Ok(v) => v,
                Err(error) => {
                    report.hold = Some(Hold::Refused(error));
                    return Ok(report);
                }
            };
            report.last_stage = Some(stage);
            report.last_report = Some(value);
            let page = match inspect(stage, report.last_report.as_ref().unwrap(), binding) {
                Ok(p) => p,
                Err(_) => {
                    report.hold = Some(Hold::InvalidReport);
                    return Ok(report);
                }
            };
            if page.count > slice.objects || page.bytes > remaining_bytes {
                report.hold = Some(Hold::Budget);
                return Ok(report);
            }
            report.object_visits += page.count;
            report.prospective_bytes += page.bytes;
            let summary = report.stages.last_mut().unwrap();
            summary.object_visits += page.count;
            summary.prospective_bytes += page.bytes;
            if start.elapsed() >= std::time::Duration::from_millis(budget.millis) {
                report.hold = Some(Hold::Budget);
                return Ok(report);
            }
            if page.held {
                report.hold = Some(Hold::StageHeld);
                return Ok(report);
            }
            if page.fresh {
                if fresh_basis.as_ref() == Some(&page.basis) {
                    report.hold = Some(Hold::NoProgress);
                    return Ok(report);
                }
                fresh_basis = Some(page.basis);
                cursor = None;
                continue;
            }
            if let Some(next) = page.next {
                if page.count == 0
                    || cursor
                        .as_ref()
                        .is_some_and(|c| c.basis == next.basis && c.after == next.after)
                {
                    report.hold = Some(Hold::NoProgress);
                    return Ok(report);
                }
                cursor = Some(next);
                continue;
            }
            report.stages.last_mut().unwrap().complete = true;
            break;
        }
    }
    report.ordered_passes_complete = true;
    Ok(report)
}
#[cfg(test)]
mod tests;
