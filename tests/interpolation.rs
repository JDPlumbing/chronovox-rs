use chronovox::{Timeline, ChronoEvent, EventKind}; // from your crate
use uvoxid::UvoxId;
use uvoxxyz::types::Cartesian;
use tdt::core::TimeDelta;
use chrono::{Utc, Duration};

/// Helper to make events at specific nanosecond offsets
fn make_event(anchor: &UvoxId, nanos: i64, kind: EventKind) -> ChronoEvent {
    let start = Utc::now();
    let end = start + Duration::nanoseconds(nanos);
    ChronoEvent {
        id: anchor.clone(),
        t: TimeDelta::between(start, end),
        kind,
    }
}

#[test]
fn interpolates_halfway_between_moves() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    // Spawn at 1000ns
    timeline.insert(make_event(&anchor, 1000, EventKind::Spawn));

    // Move at 2000ns → pos.x = 0
    timeline.insert(make_event(&anchor, 2000, EventKind::Move {
        offset: Cartesian { x: 0.0, y: 0.0, z: 0.0 },
    }));

    // Move at 3000ns → pos.x = 10
    timeline.insert(make_event(&anchor, 3000, EventKind::Move {
        offset: Cartesian { x: 10.0, y: 0.0, z: 0.0 },
    }));

    // Ask for state at 2500ns → halfway → expect x=5.0
    let state = timeline.playback_until(2500);
    let e = state.get(&anchor).unwrap();
    assert!((e.pos.x - 5.0).abs() < 1e-6, "expected ~5.0, got {}", e.pos.x);
}
