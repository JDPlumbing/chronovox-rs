// ===== Imports =====
pub use uvoxid::UvoxId;
pub use tdt::core::TimeDelta;
pub use uvoxxyz::types::Cartesian;
use std::cmp::Ordering;
use std::collections::HashMap;

// ===== Module Declarations and Re-exports =====
pub mod error;
pub mod persist;
pub use persist::{insert_event_for_entity, fetch_events_for_entity};
pub use error::{ChronovoxError, Result};

// ===== Structs, Enums, and Type Definitions =====

/// A Chronovox event: something happening at a place + time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChronoEvent {
    pub id: UvoxId,      // where
    pub t: TimeDelta,    // when
    pub kind: EventKind, // what happened
    #[serde(default)]   
    pub payload: Option<serde_json::Value>, // optional extra data
}

/// Event type / payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventKind {
    Spawn,
    Move { offset: Cartesian },
    Despawn,
    Custom(String),
}

/// A sequence of events (timeline)
#[derive(Debug, Default, Clone)]
pub struct Timeline {
    pub events: Vec<ChronoEvent>,
}

#[derive(Debug, Clone)]
pub struct EntityState {
    pub pos: Cartesian,
    pub alive: bool,
}

// ===== Implementations =====

impl Timeline {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: ChronoEvent) {
        self.events.push(event);
    }

    pub fn insert(&mut self, event: ChronoEvent) {
        self.events.push(event);
        self.events.sort(); // keep it ordered
    }

    pub fn iter_chronological(&self) -> impl Iterator<Item = &ChronoEvent> {
        self.events.iter()
    }

    pub fn query_time_range(&self, start_ns: i64, end_ns: i64) -> Vec<&ChronoEvent> {
        self.events
            .iter()
            .filter(|e| {
                let t = e.t.ticks("nanoseconds");
                t >= start_ns && t <= end_ns
            })
            .collect()
    }

    pub fn query_by_id(&self, id: &UvoxId) -> Vec<&ChronoEvent> {
        self.events.iter().filter(|e| &e.id == id).collect()
    }

    pub fn playback(&self) -> HashMap<UvoxId, EntityState> {
        let mut state = HashMap::new();
        for e in self.iter_chronological() {
            match &e.kind {
                EventKind::Spawn => {
                    state.insert(e.id.clone(), EntityState { pos: Cartesian { x: 0.0, y: 0.0, z: 0.0 }, alive: true });
                }
                EventKind::Move { offset } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos.x += offset.x;
                        s.pos.y += offset.y;
                        s.pos.z += offset.z;
                    }
                }
                EventKind::Despawn => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.alive = false;
                    }
                }
                EventKind::Custom(_) => {
                    // up to you — maybe log it or trigger hooks
                }
            }
        }
        state
    }

    /// Reconstruct state up to a given time (with interpolation for Move)
    pub fn playback_until(&self, cutoff_ns: i64) -> HashMap<UvoxId, EntityState> {
        let mut state: HashMap<UvoxId, EntityState> = HashMap::new();
        let mut last_event_by_id: HashMap<UvoxId, &ChronoEvent> = HashMap::new();

        for e in self.iter_chronological() {
            let t = e.t.ticks("nanoseconds");
            if t > cutoff_ns {
                // If cutoff is between last move and this move, interpolate
                if let Some(prev) = last_event_by_id.get(&e.id) {
                    if let (EventKind::Move { offset: prev_offset }, EventKind::Move { offset: next_offset }) =
                        (&prev.kind, &e.kind)
                    {
                        let t_prev = prev.t.ticks("nanoseconds");
                        let t_next = t;
                        let frac = (cutoff_ns - t_prev) as f64 / (t_next - t_prev) as f64;

                        if let Some(s) = state.get_mut(&e.id) {
                            s.pos = interpolate(prev_offset, next_offset, frac);
                        }
                    }
                }
                break; // stop at cutoff
            }

            // Normal event application
            match &e.kind {
                EventKind::Spawn => {
                    state.insert(
                        e.id.clone(),
                        EntityState {
                            pos: Cartesian { x: 0.0, y: 0.0, z: 0.0 },
                            alive: true,
                        },
                    );
                }
                EventKind::Move { offset } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos.x += offset.x;
                        s.pos.y += offset.y;
                        s.pos.z += offset.z;
                    }
                }
                EventKind::Despawn => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.alive = false;
                    }
                }
                EventKind::Custom(_) => {}
            }

            last_event_by_id.insert(e.id.clone(), e);
        }

        state
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl ChronoEvent {
    pub fn dummy() -> Self {
        Self {
            id: UvoxId {
                frame_id: 1,
                r_um: 0,
                lat_code: 0,
                lon_code: 0,
            },
            t: TimeDelta::from_now(),
            kind: EventKind::Spawn,
            payload: None,
        }
    }
}


// ===== Helper Functions =====

fn interpolate(prev: &Cartesian, next: &Cartesian, frac: f64) -> Cartesian {
    Cartesian {
        x: prev.x + frac * (next.x - prev.x),
        y: prev.y + frac * (next.y - prev.y),
        z: prev.z + frac * (next.z - prev.z),
    }
}

// ===== Trait Implementations =====

impl PartialEq for ChronoEvent {
    fn eq(&self, other: &Self) -> bool {
        self.t.ticks("nanoseconds") == other.t.ticks("nanoseconds")
    }
}
impl Eq for ChronoEvent {}

impl PartialOrd for ChronoEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.t.ticks("nanoseconds").cmp(&other.t.ticks("nanoseconds")))
    }
}
impl Ord for ChronoEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.t.ticks("nanoseconds").cmp(&other.t.ticks("nanoseconds"))
    }
}

// ===== Iterator Implementations =====

impl IntoIterator for Timeline {
    type Item = ChronoEvent;
    type IntoIter = std::vec::IntoIter<ChronoEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl<'a> IntoIterator for &'a Timeline {
    type Item = &'a ChronoEvent;
    type IntoIter = std::slice::Iter<'a, ChronoEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}
