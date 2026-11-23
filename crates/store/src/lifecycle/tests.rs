use super::*;
use crate::Key;
fn request() -> Request {
    Request {
        operation: Key::parse("restore").unwrap(),
        owner: Key::parse("owner").unwrap(),
        object: Object::from_bytes(b"repeat"),
        required: AckLevel::ProcessRestart,
    }
}
#[test]
fn frozen_local_wire_and_hashes_match_independent_vectors() {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/vectors/lifecycle.design-1.json"
    ))
    .unwrap();
    let store = Digest::parse(v["store"].as_str().unwrap()).unwrap();
    let terminal = Digest::parse(v["terminal"].as_str().unwrap()).unwrap();
    let context = Digest::parse(v["final_context"].as_str().unwrap()).unwrap();
    let initial = Incarnation::initial(store, request().object.clone()).unwrap();
    let frames = [
        initial.clone(),
        initial.cancelled(terminal.clone()).unwrap(),
        initial.reacquired(terminal.clone(), request()).unwrap(),
        initial
            .final_reacquired(terminal, request(), context)
            .unwrap(),
    ];
    for (frame, row) in frames.iter().zip(v["frames"].as_array().unwrap()) {
        assert_eq!(
            frame.to_json().unwrap(),
            row["canonical"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            frame.id().unwrap().as_str(),
            row["sha256"].as_str().unwrap()
        );
        assert_eq!(
            serde_json::from_slice::<Incarnation>(&frame.to_json().unwrap()).unwrap(),
            *frame
        );
    }
    assert_eq!(frames[0].placement(), frames[1].placement());
    assert_ne!(frames[0].placement(), frames[2].placement());
    assert_ne!(frames[2].placement(), frames[3].placement());
}
#[test]
fn record_transitions_refuse_mismatch_overflow_and_full_chain_without_io() {
    let first = Incarnation::initial(Digest::blob(b"store"), request().object.clone()).unwrap();
    let terminal = Digest::blob(b"terminal");
    let mut wrong = request();
    wrong.object = Object::from_bytes(b"different");
    assert_eq!(
        first.reacquired(terminal.clone(), wrong),
        Err(Error::InvalidRequest)
    );
    let mut wrong = request();
    wrong.required = AckLevel::ProviderDurable;
    assert_eq!(
        first.reacquired(terminal.clone(), wrong),
        Err(Error::InvalidRequest)
    );
    let mut head = first.clone();
    for index in 1..MAX_BIRTHS {
        head = head.cancelled(terminal.clone()).unwrap();
        assert_eq!(head.index(), index);
        assert_eq!(head.placement(), first.placement());
    }
    assert_eq!(head.cancelled(terminal.clone()), Err(Error::Budget));
    head.index = usize::MAX;
    assert_eq!(head.reacquired(terminal, request()), Err(Error::Budget));
    let mut value = serde_json::to_value(&first).unwrap();
    value["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Incarnation>(value).is_err());
    assert_eq!(first.index(), 0);
}
