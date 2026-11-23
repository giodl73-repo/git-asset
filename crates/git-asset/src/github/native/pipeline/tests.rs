use super::*;
use serde_json::json;
fn binding() -> Binding {
    let fixture = crate::github::fixture::Fixture::new();
    let proof = fixture.verify(&fixture.claims, 1000).unwrap();
    Binding {
        policy: Digest::blob(b"fixed private pipeline"),
        provenance: proof.provenance().clone(),
    }
}
fn budget() -> crate::github::placement::Budget {
    crate::github::placement::Budget {
        pages: 32,
        objects: 64,
        stream_bytes: 1024,
        millis: 60000,
    }
}
fn frame(stage: Stage, b: &Binding) -> Value {
    let state = match stage {
        Stage::Restore => "not_applicable",
        Stage::Copy => "present",
        Stage::RegisterWorking | Stage::RegisterRetained => "recorded",
        Stage::History => "released",
        Stage::Eviction => "protected",
    };
    json!({"schema":stage.schema(),"policy":b.policy,"provenance":b.provenance,
        "selection":{"schema":"tig.git-lifecycle-status.design-1","plan":Digest::blob(b"selection"),"historical":null,"desired_current":true,"source_revision":3,"observed_ref_current":true,"storage_converged":null},
        "basis":Digest::blob(b"current stage basis"),"next":null,"fresh_pass_required":false,"post_effect_hold":null,
        "side":if stage==Stage::RegisterWorking {"working"} else {"retained"},
        "outcomes":[{"state":state,"hold":null}],"prospective_bytes":0})
}
#[test]
fn fixed_order_never_claims_storage_convergence() {
    let b = binding();
    let mut seen = Vec::new();
    let r = run(&b, &budget(), |stage, cursor, _| {
        assert!(cursor.is_none());
        seen.push(stage);
        Ok(frame(stage, &b))
    })
    .unwrap();
    assert_eq!(seen, ORDER);
    assert!(r.ordered_passes_complete && r.hold.is_none());
    assert!(r.storage_converged.is_none());
    assert_eq!(r.stages.len(), 6);
}
#[test]
fn post_effect_hold_preserves_original_last_report_and_stops() {
    let b = binding();
    let mut seen = Vec::new();
    let mut saved = None;
    let r = run(&b, &budget(), |stage, _, _| {
        seen.push(stage);
        let mut v = frame(stage, &b);
        if stage == Stage::Copy {
            v["post_effect_hold"] = json!("expired");
            v["outcomes"][0]["saved"] = json!({"original":"recover this exact operation"});
            saved = Some(v.clone());
        }
        Ok(v)
    })
    .unwrap();
    assert_eq!(seen, [Stage::Restore, Stage::Copy]);
    assert!(matches!(r.hold, Some(Hold::StageHeld)));
    assert_eq!(r.last_report, saved);
    assert!(!r.ordered_passes_complete);
}
#[test]
fn pages_and_object_visits_are_aggregate_limits() {
    let b = binding();
    for pages in [true, false] {
        let mut limits = budget();
        if pages {
            limits.pages = 2;
        } else {
            limits.objects = 2;
        }
        let mut seen = Vec::new();
        let r = run(&b, &limits, |stage, _, _| {
            seen.push(stage);
            Ok(frame(stage, &b))
        })
        .unwrap();
        assert_eq!(seen, [Stage::Restore, Stage::Copy]);
        assert!(matches!(r.hold, Some(Hold::Budget)));
        assert_eq!(r.object_visits, 2);
    }
}
#[test]
fn stream_credit_is_not_reset_between_native_stages() {
    let b = binding();
    let mut limits = budget();
    limits.stream_bytes = 3;
    let mut seen = Vec::new();
    let r = run(&b, &limits, |stage, _, slice| {
        seen.push(stage);
        let mut v = frame(stage, &b);
        if stage.payload() {
            v["prospective_bytes"] = json!(slice.stream_bytes.min(2));
        }
        Ok(v)
    })
    .unwrap();
    assert!(!seen.contains(&Stage::Eviction));
    assert_eq!(r.prospective_bytes, 3);
    assert!(matches!(r.hold, Some(Hold::Budget)));
}
#[test]
fn changed_root_restarts_only_its_stage_without_client_suffix() {
    let b = binding();
    let mut calls = 0;
    let r = run(&b, &budget(), |stage, cursor, _| {
        let mut v = frame(stage, &b);
        if stage == Stage::RegisterWorking {
            assert!(cursor.is_none());
            calls += 1;
            v["basis"] = json!(Digest::blob(&[calls]));
            v["fresh_pass_required"] = json!(calls == 1);
        }
        Ok(v)
    })
    .unwrap();
    assert_eq!(calls, 2);
    assert!(r.ordered_passes_complete);
}
#[test]
fn nonadvancing_cursors_and_fresh_roots_refuse_instead_of_spinning() {
    let b = binding();
    for fresh in [false, true] {
        let r = run(&b, &budget(), |stage, _, _| {
            let mut v = frame(stage, &b);
            if fresh {
                v["fresh_pass_required"] = json!(true);
            } else {
                v["next"] =
                    json!({"basis":Digest::blob(b"cursor"),"after":Digest::blob(b"last object")});
            }
            Ok(v)
        })
        .unwrap();
        assert!(matches!(r.hold, Some(Hold::NoProgress)));
        assert_eq!(r.pages, 2);
    }
}
#[test]
fn foreign_frames_and_nested_failure_do_not_advance() {
    let b = binding();
    for foreign in [false, true] {
        let r = run(&b, &budget(), |stage, _, _| {
            let mut v = frame(stage, &b);
            if foreign {
                v["provenance"]["commit"] = json!("b".repeat(40));
            } else {
                v["outcomes"][0]["handoff"] =
                    json!({"failure":{"layer":"native","code":"uncertain"}});
            }
            Ok(v)
        })
        .unwrap();
        assert_eq!(r.pages, 1);
        assert!(!r.ordered_passes_complete);
        assert!(matches!(
            r.hold,
            Some(Hold::InvalidReport) | Some(Hold::StageHeld)
        ));
    }
}

#[test]
fn callback_refusal_and_late_receipt_keep_bounds_and_prior_evidence() {
    let b = binding();
    let r = run(&b, &budget(), |stage, _, _| {
        if stage == Stage::Copy {
            Err(AdmissionError::Unavailable)
        } else {
            Ok(frame(stage, &b))
        }
    })
    .unwrap();
    assert_eq!(r.pages, 2);
    assert_eq!(r.last_stage, Some(Stage::Restore));
    assert!(matches!(r.hold, Some(Hold::Refused(_))));
    let mut limits = budget();
    limits.millis = 100;
    let r = run(&b, &limits, |stage, _, _| {
        std::thread::sleep(std::time::Duration::from_millis(150));
        Ok(frame(stage, &b))
    })
    .unwrap();
    assert_eq!(r.pages, 1);
    assert!(r.last_report.is_some());
    assert!(matches!(r.hold, Some(Hold::Budget)));
    assert!(!r.ordered_passes_complete);
}
