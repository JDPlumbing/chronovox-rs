use chronovox::{ChronoEvent, EventKind, Timeline};
use uvoxid::UvoxId;
use uvoxxyz::types::Cartesian;
use tdt::core::TimeDelta;
use chrono::{Utc, Duration};

#[test]
fn can_create_and_store_event() {
    let mut timeline = Timeline::new();
    let anchor = UvoxId::earth(6_371_000_000, 0, 0);

    // Build a 1234 ns TimeDelta
    let start = Utc::now();
    let end = start + Duration::nanoseconds(1234);
    let td = TimeDelta::between(start, end);

    let e = ChronoEvent {
        id: anchor,
        t: td,
        kind: EventKind::Move { offset: Cartesian { x: 1.0, y: 0.0, z: 0.0 } },
    };

    timeline.push(e.clone());
    assert_eq!(timeline.events.len(), 1);

    // Check kind matches
    assert!(matches!(timeline.events[0].kind, EventKind::Move { .. }));

    // Verify nanos via ticks()
    let nanos = timeline.events[0].t.ticks("nanoseconds");
    assert!(nanos >= 1234 && nanos < 2000, "nanos = {}", nanos);
}
