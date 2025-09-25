use crate::Result;                  // Chronovox’s Result<T>
use crate::error::ChronovoxError;
use supabasic::Supabase;            // from supabasic’s lib.rs
use uuid::Uuid;
use serde_json::json;

use crate::{Timeline, ChronoEvent};

pub async fn insert_event_for_entity(
    supa: &Supabase,
    entity_id: Uuid,
    event: &ChronoEvent,
) -> Result<Uuid> {
    let event_val = json!({
        "frame_id": event.id.frame_id,
        "r_um": event.id.r_um,
        "lat_code": event.id.lat_code,
        "lon_code": event.id.lon_code,
        "ticks": event.t.ticks("nanoseconds"),
        "timestamp": chrono::Utc::now(),
        "kind": format!("{:?}", event.kind),
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

    let link_val = json!({
        "entity_id": entity_id,
        "event_id": event_id,
    });

    let link_insert: Vec<serde_json::Value> = supa
        .from("entity_events")
        .insert(json!([link_val]))
        .select("entity_id, event_id") // ✅ columns that actually exist
        .execute_typed()
        .await?;

    println!("DEBUG link_insert = {:?}", link_insert);


    Ok(event_id)
}

#[derive(Debug, serde::Deserialize)]
struct EventRow {
    event: ChronoEvent,
}

pub async fn fetch_events_for_entity(
    supa: &Supabase,
    entity_id: Uuid,
) -> Result<Timeline> {
    let rows: Vec<EventRow> = supa
        .from("entity_events")
        .select("event:events ( id, frame_id, r_um, lat_code, lon_code, ticks, kind, move_offset, payload )")
        .eq("entity_id", &entity_id.to_string())
        .execute_typed()
        .await?;

    let mut timeline = Timeline::new();
    for row in rows {
        timeline.push(row.event);
    }

    Ok(timeline)
}
