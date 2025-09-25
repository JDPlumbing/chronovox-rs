use chronovox::{Timeline, ChronoEvent, EventKind, UvoxId};
use uvoxxyz::types::Cartesian;
use tdt::core::TimeDelta;

#[test]
fn playback_handles_temp_and_pressure_changes() {
    let id = UvoxId::earth(0, 0, 0);
    let mut timeline = Timeline::new();

    // Spawn entity
    timeline.push(ChronoEvent {
        id,
        t: TimeDelta::from_ticks(0, "nanoseconds"),
        kind: EventKind::Spawn,
        payload: None,
    });

    // Raise temperature by 10°C
    timeline.push(ChronoEvent {
        id,
        t: TimeDelta::from_ticks(1, "nanoseconds"),
        kind: EventKind::TemperatureChange { delta_c: 10.0 },
        payload: None,
    });

    // Increase pressure by 500 Pascals
    timeline.push(ChronoEvent {
        id,
        t: TimeDelta::from_ticks(2, "nanoseconds"),
        kind: EventKind::PressureChange { delta_pa: 500.0 },
        payload: None,
    });

    // Teleport entity
    timeline.push(ChronoEvent {
        id,
        t: TimeDelta::from_ticks(3, "nanoseconds"),
        kind: EventKind::Teleport { new_pos: Cartesian { x: 5.0, y: -2.0, z: 1.0 } },
        payload: None,
    });

    let state_map = timeline.playback();
    let state = state_map.get(&id).expect("entity state should exist");

    assert!(state.alive);
    assert_eq!(state.temperature, 30.0); // 20 default + 10
    assert_eq!(state.pressure, 101_825.0); // 101325 default + 500
    assert_eq!(state.pos.x, 5.0);
    assert_eq!(state.pos.y, -2.0);
    assert_eq!(state.pos.z, 1.0);
}
