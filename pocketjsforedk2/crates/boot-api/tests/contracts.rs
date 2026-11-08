use pocketjs_boot_api::{
    UiAction, UiEvent,
    services::{BootParameters, BootRequest, DisplayDevice, DisplayRegion},
};
use serde_json::json;

fn parameters() -> BootParameters {
    BootParameters {
        custom: serde_json::from_value(json!({
            "kernel.args": "quiet",
            "vendor.options": {"enabled": true, "count": 7, "targets": ["a", "b"]}
        }))
        .unwrap(),
        displays: vec![DisplayDevice {
            id: "gop-main".into(),
            size: [1280, 720],
        }],
    }
}

#[test]
fn typed_parameters_and_output_regions_keep_the_existing_wire_contract() {
    let original = parameters();
    let encoded = serde_json::to_string(&original).unwrap();
    let decoded: BootParameters = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, original);
    decoded.validate().unwrap();
    assert_eq!(decoded.custom["vendor.options"]["count"].as_u64(), Some(7));
    assert_eq!(
        decoded.custom["vendor.options"]["enabled"].as_bool(),
        Some(true)
    );

    let action: UiAction = serde_json::from_value(json!({
        "type": "boot", "id": "child", "path": "\\EFI\\child.efi",
        "display_regions": [{"id":"boot-output", "display_id":"gop-main",
            "position":[84,84], "size":[1196,636]}]
    }))
    .unwrap();
    let UiAction::Boot {
        display_regions, ..
    } = action
    else {
        panic!("boot request lost its action tag");
    };
    display_regions[0].validate(&decoded.displays).unwrap();
    assert_eq!(display_regions[0].position, [84, 84]);
}

#[test]
fn output_validation_rejects_overflow_and_duplicate_regions_before_boot() {
    let parameters = parameters();
    let overflow = DisplayRegion {
        id: "overflow".into(),
        display_id: "gop-main".into(),
        position: [u32::MAX, 0],
        size: [1, 1],
    };
    assert_eq!(
        overflow.validate(&parameters.displays),
        Err("output region overflows")
    );
    let region = DisplayRegion {
        position: [0, 0],
        ..overflow
    };
    let mut opaque_runtime = 41_u64;
    let request = BootRequest {
        entry: serde_json::from_value(json!({"id":"child","name":"Child","path":"child.efi"}))
            .unwrap(),
        parameters,
        runtime: &mut opaque_runtime,
        display_regions: vec![region.clone(), region],
    };
    assert_eq!(request.validate(), Err("duplicate output region"));
    assert_eq!(*request.runtime, 41);
}

#[test]
fn unknown_battery_state_is_preserved_without_fabricating_a_percentage() {
    let event: UiEvent = serde_json::from_value(json!({
        "type":"power_state", "battery_percent":null, "external_power":false
    }))
    .unwrap();
    assert!(matches!(
        event,
        UiEvent::PowerState {
            battery_percent: None,
            external_power: Some(false), ..
        }
    ));
    assert_eq!(
        serde_json::to_value(event).unwrap()["battery_percent"],
        json!(null)
    );
}

#[test]
fn unavailable_external_power_remains_distinct_from_disconnected() {
    let event: UiEvent = serde_json::from_value(json!({
        "type":"power_state", "battery_percent":null, "external_power":null
    })).unwrap();
    assert!(matches!(event,UiEvent::PowerState { battery_percent:None,external_power:None,.. }));
    assert_eq!(serde_json::to_value(event).unwrap()["external_power"],json!(null));
}

#[test]
fn power_events_keep_signed_battery_measurements_and_accept_legacy_senders() {
    for current in [1620,-420,0] {
        let value=json!({"type":"power_state","battery_percent":72,"external_power":true,
            "power":{"charging":current>0,"voltage_mv":4152,"current_ma":current}});
        let event:UiEvent=serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(event).unwrap(),value);
    }
    let event:UiEvent=serde_json::from_value(json!({"type":"power_state",
        "battery_percent":null,"external_power":true})).unwrap();
    assert!(matches!(event,UiEvent::PowerState {power:p,..} if p==Default::default()));
}
