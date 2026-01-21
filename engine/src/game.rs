use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::card::{Card, Deck};
use crate::ELoggingVerbosity;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameStep 
{
    StartTurn,
    Untap,
    Upkeep,
    Draw,
    Main,
    Combat,
    EndTurn,
    GameOver,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone
{
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StepCommand
{
    StepPhase,       // "s"
    StepTurn,        // "t"
    RunGame,         // "g"
    RunDeck,         // "d"
    RunAll,          // "r"
    Quit,            // "q"
    Invalid,         // anything else
}

pub struct ProgramState 
{
    pub step_mode: StepCommand,
}

impl ProgramState
{
    pub fn new() -> Self
    {
        ProgramState
        {
            step_mode: StepCommand::StepPhase,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player
{
    pub life: i32,
    pub zones: HashMap<Zone, Vec<Card>>,
}

impl Player
{
    pub fn new(deck: &Deck) -> Self
    {
        let mut rng = thread_rng();
        let mut library = deck.cards.clone();
        library.shuffle(&mut rng);

        let mut hand = Vec::new();
        for _ in 0..7
        {
            if let Some(card) = library.pop()
            {
                hand.push(card);
            }
        }

        let mut zones = HashMap::new();
        zones.insert(Zone::Library, library);
        zones.insert(Zone::Hand, hand);
        zones.insert(Zone::Battlefield, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());
        zones.insert(Zone::Exile, Vec::new());

        Player
        {
            life: 20,
            zones,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState 
{
    pub players: Vec<Player>,
    pub current_player_index: usize,
    pub turns: u32,
    pub step: GameStep,
}

impl GameState 
{
    pub fn new(player_count: usize, deck: &Deck) -> Self 
    {
        let mut players = Vec::new();
        for _ in 0..player_count.max(2) // Minimum 2 players
        {
            players.push(Player::new(deck));
        }

        GameState
        {
            players,
            current_player_index: 0,
            turns: 0,
            step: GameStep::StartTurn,
        }
    }

    pub fn new_default() -> Self {
        let deck = Deck::example();
        Self::new(2, &deck) // Default 2 players
    }

    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_index]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    pub fn other_players(&self) -> Vec<&Player> {
        self.players.iter().enumerate()
            .filter(|(i, _)| *i != self.current_player_index)
            .map(|(_, p)| p)
            .collect()
    }

    pub fn other_players_mut(&mut self) -> Vec<&mut Player> {
        let current_idx = self.current_player_index;
        self.players.iter_mut().enumerate()
            .filter(|(i, _)| *i != current_idx)
            .map(|(_, p)| p)
            .collect()
    }

    // Backward compatibility: access current player's zones
    pub fn zones(&self) -> &HashMap<Zone, Vec<Card>> {
        &self.current_player().zones
    }

    pub fn zones_mut(&mut self) -> &mut HashMap<Zone, Vec<Card>> {
        &mut self.current_player_mut().zones
    }

    pub fn life(&self) -> i32 {
        self.current_player().life
    }

    pub fn set_life(&mut self, life: i32) {
        self.current_player_mut().life = life;
    }
}

impl GameState 
{
    pub fn step(&mut self)
    {
        match self.step
        {
            GameStep::StartTurn =>
            {
                self.turns += 1;
                self.step = GameStep::Untap;
            }

            GameStep::Untap =>
            {
                // Untap all tappable cards
                {
                    let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                    for card in battlefield.iter_mut()
                    {
                        if crate::tappable::is_tapped(card)
                        {
                            crate::tappable::set_tapped(card, false);
                        }
                    }
                }

                self.step = GameStep::Upkeep;
            }

            GameStep::Upkeep =>
            {
                // Remove summoning sickness from creatures that have it
                let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                for card in battlefield.iter_mut()
                {
                    crate::creature::set_summoning_sickness(card, false);
                }

                self.step = GameStep::Draw;
            }

            GameStep::Draw =>
            {
                let card = 
                {
                    let library = self.zones_mut().get_mut(&Zone::Library).unwrap();
                    library.pop()
                };

                if let Some(card) = card 
                {
                    let hand = self.zones_mut().get_mut(&Zone::Hand).unwrap();
                    hand.push(card);
                    self.step = GameStep::Main;
                } 
                else 
                {
                    self.step = GameStep::GameOver;
                }
            }

            GameStep::Main =>
            {
                // Play up to one land
                {
                    let card_option =
                    {
                        let hand = self.zones_mut().get_mut(&Zone::Hand).unwrap();
                        if let Some(pos) = hand.iter().position(|c| c.is_type(crate::card::CardType::Land))
                        {
                            Some(hand.remove(pos))  // hand borrow ends here
                        }
                        else
                        {
                            None
                        }
                    };

                    if let Some(card) = card_option
                    {
                        let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                        battlefield.push(card);
                    }
                }

                // Cast as many creatures as possible until there is no more mana
                loop
                {
                    // Count available untapped lands as available mana
                    let available_mana = self.zones().get(&Zone::Battlefield).unwrap().iter().filter(|card| 
                        card.is_type(crate::card::CardType::Land) && !crate::tappable::is_tapped(card)).count() as u32;

                    // Find first castable creature in hand
                    let cast_pos = 
                    {
                        let hand = self.zones().get(&Zone::Hand).unwrap();
                        hand.iter().position(|card| crate::creature::is_creature(card) && card.cost <= available_mana)
                    };

                    if let Some(pos) = cast_pos
                    {
                        // Remove card first
                        let mut card = 
                        {
                            let hand = self.zones_mut().get_mut(&Zone::Hand).unwrap();
                            hand.remove(pos)
                        };

                        vlog!(ELoggingVerbosity::Verbose, "Cast {}", card.name);

                        // Newly cast creatures have summoning sickness
                        crate::creature::set_summoning_sickness(&mut card, true);

                        // Tap lands to pay for the creature's cost
                        let mut need = card.cost;
                        {
                            let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                            for b in battlefield.iter_mut().filter(|c| c.is_type(crate::card::CardType::Land) && !crate::tappable::is_tapped(c)) 
                            {
                                if need == 0 
                                { 
                                    break; 
                                }
                                crate::tappable::set_tapped(b, true);
                                need -= 1;
                            }
                        }

                        // Put the card onto the battlefield
                        let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                        battlefield.push(card);
                    }
                    else
                    {
                        // Nothing more can be cast
                        break;
                    }
                }

                self.step = GameStep::Combat;
            }

            GameStep::Combat =>
            {
                let battlefield = self.zones_mut().get_mut(&Zone::Battlefield).unwrap();
                let mut damage = 0;
                for card in battlefield.iter_mut().filter(|card| card.is_type(crate::card::CardType::Creature) && !crate::creature::has_summoning_sickness(card) && !crate::tappable::is_tapped(card))
                {
                    damage += crate::creature::creature_stats(card).map(|stat| stat.power as u32).unwrap_or(0);
                    crate::tappable::set_tapped(card, true);
                }

                // Apply damage to all other players
                for other_player in self.other_players_mut() {
                    other_player.life -= damage as i32;
                }

                // Check if any player has lost
                let anyone_dead = self.players.iter().any(|p| p.life <= 0);
                if anyone_dead {
                    self.step = GameStep::GameOver;
                } else {
                    self.step = GameStep::EndTurn;
                }
            }

            GameStep::EndTurn =>
            {
                // Advance to next player
                self.current_player_index = (self.current_player_index + 1) % self.players.len();
                self.step = GameStep::StartTurn;
            }

            GameStep::GameOver =>
            {
                // Do nothing
            }
        }
    }

    pub fn is_game_over(&self) -> bool
    {
        self.step == GameStep::GameOver
    }

    pub fn describe(&self, verbose: bool)
    {
        println!("Turn: {}", self.turns);
        println!("Step: {:?}", self.step);
        println!("Life: {}", self.life());

        if verbose 
        {
            self.describe_verbose();
        } 
        else 
        {
            self.describe_summary();
        }
    }

    pub fn describe_summary(&self)
    {
        // Print only zone counts
        for zone in &[Zone::Hand, Zone::Battlefield, Zone::Library, Zone::Graveyard, Zone::Exile]
        {
            let cards = self.zones().get(zone).unwrap();
            println!("{:?}: {} cards", zone, cards.len());
        }
    }

    pub fn describe_verbose(&self)
    {
        for zone in &[Zone::Hand, Zone::Battlefield, Zone::Library, Zone::Graveyard]
        {
            let cards = self.zones().get(zone).unwrap();
            if cards.is_empty() && (*zone == Zone::Battlefield || *zone == Zone::Graveyard)
            {
                continue;
            }

            println!("{:?}: ({} cards)", zone, cards.len());

            match zone
            {
                Zone::Library =>
                {
                    // Show library cards grouped by count
                    let mut card_groups: HashMap<String, u32> = HashMap::new();
                    for card in cards.iter()
                    {
                        *card_groups.entry(card.name.clone()).or_insert(0) += 1;
                    }

                    for (name, count) in card_groups.iter()
                    {
                        println!("  {} x{}", name, count);
                    }
                }
                Zone::Hand =>
                {
                    // Print hand cards grouped by count in an inline list
                    let mut groups: HashMap<String, u32> = HashMap::new();
                    for card in cards.iter()
                    {
                        *groups.entry(card.name.clone()).or_insert(0) += 1;
                    }

                    let mut items: Vec<(String, u32)> = groups.into_iter().collect();
                    items.sort_by(|a, b| a.0.cmp(&b.0));

                    let mut parts: Vec<String> = Vec::new();
                    for (name, count) in items.iter()
                    {
                        if *count > 1
                        {
                            parts.push(format!("{} x{}", name, count));
                        }
                        else
                        {
                            parts.push(name.clone());
                        }
                    }

                    if !parts.is_empty()
                    {
                        println!("  {}", parts.join(", "));
                    }
                }
                Zone::Battlefield =>
                {
                    // Group identical cards together with counts (use owned String keys)
                    let mut card_groups: HashMap<String, (String, u8, u8, bool, bool, u32)> = HashMap::new();
                    for card in cards.iter()
                    {
                        let power = crate::creature::creature_stats(card).map(|s| s.power).unwrap_or(0);
                        let toughness = crate::creature::creature_stats(card).map(|s| s.toughness).unwrap_or(0);
                        let is_creature = crate::creature::is_creature(card);
                        let is_sick = crate::creature::has_summoning_sickness(card);

                        let uniquename = if is_creature && is_sick
                        {
                            format!("{} (sick)", card.name)
                        }
                        else
                        {
                            card.name.clone()
                        };

                        card_groups.entry(uniquename)
                            .and_modify(|(_, _, _, _, _, count)| *count += 1)
                            .or_insert((card.name.clone(), power, toughness, is_creature, is_sick, 1));
                    }

                    for (_uniquename, (name, power, toughness, is_creature, is_sick, count)) in card_groups.iter()
                    {
                        if *is_creature
                        {
                            if *count > 1
                            {
                                println!("  {}: {}/{} x{} ({})", name, power, toughness, count, is_sick.then(|| "sick").unwrap_or("ready"));
                            }
                            else
                            {
                                println!("  {}: {}/{} ({})", name, power, toughness, is_sick.then(|| "sick").unwrap_or("ready"));
                            }
                        }
                        else
                        {
                            if *count > 1
                            {
                                println!("  {} x{}", name, count);
                            }
                            else
                            {
                                println!("  {}", name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::card::{grizzly_bears, forest};
    use crate::creature;

    #[test]
    fn creature_without_sickness_deals_damage()
    {
        let mut battlefield = Vec::new();
        let mut g = grizzly_bears();
        creature::add_creature_fragment(&mut g, 2, 2);
        creature::set_summoning_sickness(&mut g, false);
        battlefield.push(g);

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Hand, Vec::new());
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Combat };
        gs.step();
        assert_eq!(gs.life, 18);
    }

    #[test]
    fn creature_with_sickness_does_not_deal_damage()
    {
        let mut battlefield = Vec::new();
        let mut g = grizzly_bears();
        creature::set_summoning_sickness(&mut g, true);
        battlefield.push(g);

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Hand, Vec::new());
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Combat };
        gs.step();
        assert_eq!(gs.life, 20);
    }

    #[test]
    fn summoning_sickness_cleared_on_upkeep()
    {
        let mut battlefield = Vec::new();
        let mut g = grizzly_bears();
        creature::set_summoning_sickness(&mut g, true);
        battlefield.push(g);

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Hand, Vec::new());
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Upkeep };
        gs.step();
        let bf = gs.zones.get(&Zone::Battlefield).unwrap();
        assert!(!crate::creature::has_summoning_sickness(&bf[0]));
    }

    #[test]
    fn play_one_land_if_available()
    {
        let library = Vec::new();
        let mut hand = Vec::new();
        hand.push(forest());

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Library, library);
        zones.insert(Zone::Hand, hand);
        zones.insert(Zone::Battlefield, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Main };
        gs.step();

        assert_eq!(gs.zones.get(&Zone::Battlefield).unwrap().len(), 1);
    }

    #[test]
    fn play_as_many_creatures_as_possible()
    {
        // Start with 4 lands available and two creatures in hand (cost 2 each)
        let mut hand = Vec::new();
        hand.push(grizzly_bears());
        hand.push(grizzly_bears());

        let mut battlefield = Vec::new();
        for _ in 0..4 
        {
            battlefield.push(forest());
        }

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Hand, hand);
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Main };
        gs.step();

        // Only ONE creature should be cast per main phase (4 lands available, but can only cast 1 creature)
        assert_eq!(gs.zones.get(&Zone::Battlefield).unwrap().len(), 5); // 4 lands + 1 creature
        // Verify we have the 4 lands still on battlefield
        assert_eq!(gs.zones.get(&Zone::Battlefield).unwrap().iter().filter(|c| c.is_type(crate::card::CardType::Land)).count(), 4);
        // One creature should be in hand still
        assert_eq!(gs.zones.get(&Zone::Hand).unwrap().len(), 1);
    }

    #[test]
    fn multi_turn_summoning_sickness_flow()
    {
        // Hand: 2x Forest + Grizzly, Battlefield: 1x Forest (to give us 2 mana for grizzly)
        // Library: 2x Forest (for subsequent draws)
        // This ensures we can play another land and cast the grizzly in the first main phase
        let mut hand = Vec::new();
        hand.push(forest());
        hand.push(forest());
        hand.push(grizzly_bears());

        let mut battlefield = Vec::new();
        battlefield.push(forest());

        let mut library = Vec::new();
        library.push(forest());
        library.push(forest());

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Library, library);
        zones.insert(Zone::Hand, hand);
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::StartTurn };

        // Turn 1: StartTurn -> Untap -> Upkeep -> Draw -> Main -> Combat
        gs.step(); // StartTurn -> Untap
        gs.step(); // Untap -> Upkeep
        gs.step(); // Upkeep -> Draw (draws a forest)
        gs.step(); // Draw -> Main
        gs.step(); // Main -> Combat (plays 1 land, casts grizzly with 2 mana total, gives it summoning sickness)
        gs.step(); // Combat should NOT deal damage because creature is sick
        assert_eq!(gs.life, 20, "Creature with summoning sickness should not deal damage on the turn it was cast");

        // Continue to EndTurn -> StartTurn -> Untap -> Upkeep (for turn 2)
        gs.step(); // Combat -> EndTurn
        gs.step(); // EndTurn -> StartTurn
        gs.step(); // StartTurn -> Untap
        gs.step(); // Untap -> Upkeep (clears sickness)

        // Advance to Combat of second turn
        gs.step(); // Upkeep -> Draw (draws another forest)
        gs.step(); // Draw -> Main
        gs.step(); // Main -> Combat
        gs.step(); // Combat should now deal damage
        assert!(gs.life < 20, "Creature should deal damage after sickness cleared on upkeep");
    }

    #[test]
    fn casting_taps_forests_used_for_payment()
    {
        // Battlefield: 2x Forest (untapped). Hand: Grizzly Bears (cost 2). Main phase.
        let mut hand = Vec::new();
        hand.push(grizzly_bears());

        let mut battlefield = Vec::new();
        battlefield.push(forest());
        battlefield.push(forest());

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Hand, hand);
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Main };
        gs.step();

        // After casting, a grizzly should be on the battlefield and two forests should be tapped
        let bf = gs.zones.get(&Zone::Battlefield).unwrap();
        assert_eq!(bf.iter().filter(|c| c.is_type(crate::card::CardType::Land)).count(), 2);
        assert_eq!(bf.iter().filter(|c| c.is_type(crate::card::CardType::Creature)).count(), 1);
        let tapped_lands = bf.iter().filter(|c| c.is_type(crate::card::CardType::Land) && crate::tappable::is_tapped(c)).count();
        assert_eq!(tapped_lands, 2, "Both forests used to pay should be tapped");
    }

    #[test]
    fn untap_phase_clears_tapped_state()
    {
        let mut battlefield = Vec::new();
        let mut f = forest();
        crate::tappable::set_tapped(&mut f, true);
        battlefield.push(f);

        let mut zones = std::collections::HashMap::new();
        zones.insert(Zone::Battlefield, battlefield);
        zones.insert(Zone::Hand, Vec::new());
        zones.insert(Zone::Library, Vec::new());
        zones.insert(Zone::Graveyard, Vec::new());

        let mut gs = GameState { zones, life: 20, turns: 0, step: GameStep::Untap };
        gs.step();

        let bf = gs.zones.get(&Zone::Battlefield).unwrap();
        assert!(!crate::tappable::is_tapped(&bf[0]));
    }
}
