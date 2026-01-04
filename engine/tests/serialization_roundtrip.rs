use engine::{GameState, Zone, CardType};

#[test]
fn game_state_roundtrip_serialization() {
    // Create a default game state, serialize to JSON, then deserialize back
    let gs = GameState::new_default();

    let json = serde_json::to_string(&gs).expect("serialize GameState");

    let gs2: GameState = serde_json::from_str(&json).expect("deserialize GameState");

    // Basic structural checks
    assert_eq!(gs.life, gs2.life);
    assert_eq!(gs.turns, gs2.turns);
    assert_eq!(gs.step, gs2.step);

    for zone in &[Zone::Library, Zone::Hand, Zone::Battlefield, Zone::Graveyard] {
        assert_eq!(gs.zones.get(zone).unwrap().len(), gs2.zones.get(zone).unwrap().len());
    }

    // Spot-check a sample card if library is non-empty
    let lib = gs.zones.get(&Zone::Library).unwrap();
    let lib2 = gs2.zones.get(&Zone::Library).unwrap();
    if !lib.is_empty() {
        assert_eq!(lib[0].name, lib2[0].name);
        assert_eq!(lib[0].is_type(CardType::Creature), lib2[0].is_type(CardType::Creature));
    }
}
