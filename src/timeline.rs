use std::cmp::Ordering;
use std::collections::HashMap;
use crate::{ChronoEvent, EventKind, UvoxId, Cartesian};

#[derive(Debug, Default, Clone)]
pub struct Timeline {
    pub events: Vec<ChronoEvent>,
}

#[derive(Debug, Clone)]
pub struct EntityState {
    pub pos: Cartesian,
    pub alive: bool,
    pub temperature: f64, // °C
    pub pressure: f64,    // Pascals
}


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
                // === Core Lifecycle ===
                EventKind::Spawn => {
                    state.insert(
                        e.id.clone(),
                        EntityState {
                            pos: Cartesian { x: 0.0, y: 0.0, z: 0.0 },
                            alive: true,
                            temperature: 20.0, // default room temp
                            pressure: 101_325.0, // default 1 atm
                        },
                    );
                }

                EventKind::Despawn => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.alive = false;
                    }
                }

                // === Movement ===
                EventKind::Move { offset } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos.x += offset.x;
                        s.pos.y += offset.y;
                        s.pos.z += offset.z;
                    }
                }
                EventKind::Teleport { new_pos } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos = new_pos.clone();
                    }
                }

                // === Environment ===
                EventKind::TemperatureChange { delta_c } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.temperature += delta_c;
                    }
                }
                EventKind::PressureChange { delta_pa } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pressure += delta_pa;
                    }
                }

                EventKind::Radiation { dose: _ } => {
                    // TODO
                }
                EventKind::Shock { g: _ } => {
                    // TODO
                }

                // === Material / Integrity ===
                EventKind::Degrade { rate: _ } => {
                    // TODO
                }
                EventKind::Leak { severity: _ } => {
                    // TODO
                }
                EventKind::Fracture { plane: _ } => {
                    // TODO
                }

                // === Interactions ===
                EventKind::Bond { with: _ } => {
                    // TODO: mark bonded state
                }
                EventKind::Unbond { from: _ } => {
                    // TODO: mark bond broken
                }
                EventKind::Transfer { to: _, what: _, amount: _ } => {
                    // TODO: transfer logic
                }

                // === Wild Card ===
                EventKind::Custom(_) => {
                    // maybe log it or trigger hooks
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
                // Handle interpolation between two Move events
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
                // === Core Lifecycle ===
                EventKind::Spawn => {
                    state.insert(
                        e.id.clone(),
                        EntityState {
                            pos: Cartesian { x: 0.0, y: 0.0, z: 0.0 },
                            alive: true,
                            temperature: 20.0,   // default °C
                            pressure: 101_325.0, // default Pa
                        },
                    );
                }
                EventKind::Despawn => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.alive = false;
                    }
                }

                // === Movement ===
                EventKind::Move { offset } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos.x += offset.x;
                        s.pos.y += offset.y;
                        s.pos.z += offset.z;
                    }
                }
                EventKind::Teleport { new_pos } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pos = new_pos.clone();
                    }
                }

                // === Environment ===
                EventKind::TemperatureChange { delta_c } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.temperature += delta_c;
                    }
                }
                EventKind::PressureChange { delta_pa } => {
                    if let Some(s) = state.get_mut(&e.id) {
                        s.pressure += delta_pa;
                    }
                }

                EventKind::Radiation { dose: _ } => {
                    // TODO: accumulate radiation dose
                }
                EventKind::Shock { g: _ } => {
                    // TODO: apply shock/damage
                }

                // === Material / Integrity ===
                EventKind::Degrade { rate: _ } => {
                    // TODO: mark progressive degradation
                }
                EventKind::Leak { severity: _ } => {
                    // TODO: track fluid/gas loss
                }
                EventKind::Fracture { plane: _ } => {
                    // TODO: mark fracture in state
                }

                // === Interactions ===
                EventKind::Bond { with: _ } => {
                    // TODO: link entities
                }
                EventKind::Unbond { from: _ } => {
                    // TODO: unlink entities
                }
                EventKind::Transfer { to: _, what: _, amount: _ } => {
                    // TODO: handle resource transfer
                }

                // === Wild Card ===
                EventKind::Custom(_) => {
                    // TODO: maybe just record it
                }
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

// ===== Helper =====

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

// ===== Iterators =====

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
