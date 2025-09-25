use crate::Result; // Chronovox’s Result<T>
use crate::error::ChronovoxError;
use supabasic::Supabase;
use uuid::Uuid;
use serde_json::json;

use crate::{Timeline, ChronoEvent, EventKind};
use uvoxid::UvoxId;
use tdt::core::TimeDelta;

pub async fn insert_event_for_entity(
    supa: &Supabase,
    entity_id: Uuid,
    event: &ChronoEvent,
) -> Result<Uuid> {
    let event_val = json!({
        "entity_id": entity_id,
        "frame_id": event.id.frame_id as i64,
        "r_um": event.id.r_um as i64,
        "lat_code": event.id.lat_code as i64,
        "lon_code": event.id.lon_code as i64,
        "ticks": event.t.ticks("nanoseconds"),
        "timestamp": chrono::Utc::now(),
        "kind": format!("{:?}", event.kind),
        "move_offset": event.payload, // TODO: maybe split later
        "payload": event.payload,
    });

    let inserted: Vec<serde_json::Value> = supa
        .from("events")
        .insert(json!([event_val]))
        .select("id")
        .execute_typed()
        .await?;

    println!("DEBUG inserted = {:?}", inserted);

    let event_id = inserted
        .get(0)
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ChronovoxError::MissingField("id".into()))?
        .parse::<Uuid>()
        .map_err(|_| ChronovoxError::MissingField("id parse".into()))?;

    Ok(event_id)
}

#[derive(Debug, serde::Deserialize)]
struct EventRowDb {
    id: Uuid,
    frame_id: i64,
    r_um: i64,
    lat_code: i64,
    lon_code: i64,
    ticks: i64,
    kind: String,
    move_offset: Option<serde_json::Value>,
    payload: Option<serde_json::Value>,
}

impl EventRowDb {
    fn into_event(self) -> ChronoEvent {
        ChronoEvent {
            id: UvoxId {
                frame_id: self.frame_id as u64,
                r_um: self.r_um as u64,
                lat_code: self.lat_code, // ✅ keep as i64
                lon_code: self.lon_code, // ✅ keep as i64
            },
            t: TimeDelta::from_ticks(self.ticks, "nanoseconds"),
            kind: match self.kind.as_str() {
                "Spawn" => EventKind::Spawn,
                "Despawn" => EventKind::Despawn,
                other => EventKind::Custom(other.to_string()),
            },
            payload: self.payload.or(self.move_offset),
        }
    }
}


pub async fn fetch_events_for_entity(
    supa: &Supabase,
    entity_id: Uuid,
) -> Result<Timeline> {
    let rows: Vec<EventRowDb> = supa
        .from("events")
        .select("id, frame_id, r_um, lat_code, lon_code, ticks, kind, move_offset, payload")
        .eq("entity_id", &entity_id.to_string())
        .execute_typed()
        .await?;

    let mut timeline = Timeline::new();
    for row in rows {
        timeline.push(row.into_event());
    }

    Ok(timeline)
}
