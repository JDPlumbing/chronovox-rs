use chronovox::{ChronoEvent, EventKind, Timeline, EntityState};
use uvoxid::UvoxId;
use uvoxxyz::types::Cartesian;
use tdt::core::TimeDelta;
use chrono::{Utc, Duration};

fn make_event(anchor: &UvoxId, nanos: i64, kind: EventKind) -> ChronoEvent {
    let start = Utc::now();
    let end = start + Duration::nanoseconds(nanos);
    ChronoEvent {
        id: anchor.clone(),
        t: TimeDelta::between(start, end),
        kind,
        payload: None,
    }
}

#[test]
fn playback_until_stops_at_cutoff() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    // Insert events out-of-order (Timeline::insert sorts them)
    timeline.insert(make_event(&anchor, 1000, EventKind::Spawn));
    timeline.insert(make_event(&anchor, 3000, EventKind::Despawn));
    timeline.insert(make_event(&anchor, 2000, EventKind::Move {
        offset: Cartesian { x: 1.0, y: 0.0, z: 0.0 },
    }));

    // At 1500ns: only Spawn applied
    let state_early = timeline.playback_until(1500);
    let e1 = state_early.get(&anchor).unwrap();
    assert_eq!(e1.pos.x, 0.0);
    assert!(e1.alive);

    // At 2500ns: Spawn + Move applied
    let state_mid = timeline.playback_until(2500);
    let e2 = state_mid.get(&anchor).unwrap();
    assert_eq!(e2.pos.x, 1.0);
    assert!(e2.alive);

    // At 4000ns: Spawn + Move + Despawn applied
    let state_end = timeline.playback_until(4000);
    let e3 = state_end.get(&anchor).unwrap();
    assert_eq!(e3.pos.x, 1.0);
    assert!(!e3.alive);
}
