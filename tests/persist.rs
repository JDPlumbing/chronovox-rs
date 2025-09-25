// tests/persist.rs
use supabasic::Supabase;
use supabasic::entities::Entity;
use uuid::Uuid;
use dotenvy::dotenv;
use std::env;
use chrono::Utc;

use chronovox::{insert_event_for_entity, fetch_events_for_entity, ChronoEvent, EventKind, Timeline};
use uvoxid::UvoxId;
use tdt::core::TimeDelta;
use serde_json::json;


#[tokio::test]
async fn test_insert_and_fetch_event() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SUPABASE_URL").unwrap();
    let key = std::env::var("SUPABASE_KEY").unwrap();
    let supa = supabasic::Supabase::new(&url, &key);

    // 1. Create entity
    let entity_id = supa.create_entity("test-entity").await.unwrap();
    println!("Created entity_id = {}", entity_id);

    // 2. Insert event
    let event = chronovox::ChronoEvent::dummy(); // use your helper
    let event_id = chronovox::insert_event_for_entity(&supa, entity_id, &event)
        .await
        .unwrap();
    println!("Inserted event_id = {}", event_id);

    // 3. Fetch events
    let timeline = chronovox::fetch_events_for_entity(&supa, entity_id)
        .await
        .unwrap();
    println!("Fetched timeline with {} events", timeline.len());

    assert!(timeline.len() > 0, "Timeline should contain at least one event");
}
