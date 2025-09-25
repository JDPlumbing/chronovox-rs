// tests/persist.rs
use chronovox::{insert_event_for_entity, fetch_events_for_entity};
use supabasic::Supabase;
use uuid::Uuid;
use dotenvy::dotenv;
use std::env;

#[tokio::test]
async fn test_insert_and_fetch_event() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let url = env::var("SUPABASE_URL")?;
    let key = env::var("SUPABASE_KEY")?;
    let supa = Supabase::new(&url, &key);

    // Step 1: Create a dummy entity (use supabasic’s entity helper)
    let entity_id = supa.create_entity("persist-test-entity").await?;
    println!("Created entity_id = {}", entity_id);

    // Step 2: Make a dummy ChronoEvent
    let dummy_event = chronovox::ChronoEvent::dummy(); // assume you have a helper, otherwise stub one

    // Step 3: Insert event for entity
    let event_id = insert_event_for_entity(&supa, entity_id, &dummy_event).await?;
    println!("Inserted event_id = {}", event_id);

    // Step 4: Fetch timeline back
    let timeline = fetch_events_for_entity(&supa, entity_id).await?;
    println!("Fetched timeline with {} events", timeline.len());

    assert!(
        timeline.len() > 0,
        "Expected at least one event in timeline, got 0"
    );

    Ok(())
}
