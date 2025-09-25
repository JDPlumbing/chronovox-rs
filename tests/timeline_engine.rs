use chronovox::{ChronoEvent, EventKind, Timeline};
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
fn timeline_orders_events_correctly() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    // Insert out-of-order
    let e1 = make_event(&anchor, 5000, EventKind::Custom("middle".into()));
    let e2 = make_event(&anchor, 1000, EventKind::Custom("early".into()));
    let e3 = make_event(&anchor, 9000, EventKind::Custom("late".into()));

    timeline.insert(e1);
    timeline.insert(e2);
    timeline.insert(e3);

    let labels: Vec<String> = timeline
        .iter_chronological()
        .map(|e| match &e.kind {
            EventKind::Custom(s) => s.clone(),
            _ => "?".into(),
        })
        .collect();

    assert_eq!(labels, vec!["early", "middle", "late"]);
}

#[test]
fn query_time_range_filters_properly() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    let e1 = make_event(&anchor, 1000, EventKind::Spawn);
    let e2 = make_event(&anchor, 2000, EventKind::Move { offset: Cartesian { x: 1.0, y: 0.0, z: 0.0 } });
    let e3 = make_event(&anchor, 5000, EventKind::Despawn);

    timeline.insert(e1);
    timeline.insert(e2);
    timeline.insert(e3);

    // Only the first two should fall within this range
    let filtered = timeline.query_time_range(0, 3000);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn query_by_id_filters_properly() {
    let mut timeline = Timeline::new();
    let house = UvoxId::earth(6_371_000_000, 45_000_000, 0);
    let tree = UvoxId::earth(6_371_000_000, 46_000_000, 0);

    let e1 = make_event(&house, 1000, EventKind::Spawn);
    let e2 = make_event(&tree, 2000, EventKind::Spawn);

    timeline.insert(e1);
    timeline.insert(e2);

    let house_events = timeline.query_by_id(&house);
    assert_eq!(house_events.len(), 1);
    assert!(matches!(house_events[0].kind, EventKind::Spawn));
}
