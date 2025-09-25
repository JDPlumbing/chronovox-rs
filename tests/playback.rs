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
    }
}

#[test]
fn playback_applies_spawn_move_despawn() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    timeline.insert(make_event(&anchor, 1000, EventKind::Spawn));
    timeline.insert(make_event(&anchor, 2000, EventKind::Move { offset: Cartesian { x: 1.0, y: 2.0, z: 0.0 } }));
    timeline.insert(make_event(&anchor, 3000, EventKind::Despawn));

    let state = timeline.playback();

    let entity = state.get(&anchor).expect("entity missing");
    assert_eq!(entity.pos.x, 1.0);
    assert_eq!(entity.pos.y, 2.0);
    assert_eq!(entity.pos.z, 0.0);
    assert_eq!(entity.alive, false);
}
