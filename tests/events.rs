use chronovox::{ChronoEvent, EventKind, UvoxId, TimeDelta, Cartesian};
use uuid::Uuid;
use serde_json;

#[test]
fn eventkind_serialization_roundtrip() {
    let base_id = UvoxId {
        frame_id: 0,
        r_um: 1000,
        lat_code: 42,
        lon_code: -42,
    };
    let t = TimeDelta::from_now();

    let variants = vec![
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Spawn, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Despawn, payload: None },
        ChronoEvent {
            id: base_id,
            t: t.clone(),
            kind: EventKind::Move { offset: Cartesian { x: 1.0, y: 2.0, z: 3.0 } },
            payload: None,
        },
        ChronoEvent {
            id: base_id,
            t: t.clone(),
            kind: EventKind::Teleport { new_pos: Cartesian { x: -1.0, y: 0.5, z: 42.0 } },
            payload: None,
        },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::TemperatureChange { delta_c: 100.0 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::PressureChange { delta_pa: 101325.0 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Radiation { dose: 0.05 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Shock { g: 9.81 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Degrade { rate: 0.01 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Leak { severity: 0.5 }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Fracture { plane: "X-Y".to_string() }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Bond { with: Uuid::new_v4() }, payload: None },
        ChronoEvent { id: base_id, t: t.clone(), kind: EventKind::Unbond { from: Uuid::new_v4() }, payload: None },
        ChronoEvent {
            id: base_id,
            t: t.clone(),
            kind: EventKind::Transfer { to: Uuid::new_v4(), what: "water".into(), amount: 3.14 },
            payload: None,
        },
        ChronoEvent { id: base_id, t, kind: EventKind::Custom("Magic".into()), payload: Some(serde_json::json!({"foo": "bar"})) },
    ];

    for e in variants {
        let json = serde_json::to_string(&e).expect("serialize");
        let back: ChronoEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(format!("{:?}", e.kind), format!("{:?}", back.kind), "kind mismatch");
    }
}
