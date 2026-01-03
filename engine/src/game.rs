use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

use crate::card::{Card, Deck};
use crate::creature;
use crate::ELoggingVerbosity;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameStep 
{
    StartTurn,
    Draw,
    Main,
    Combat,
    EndTurn,
    GameOver,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

pub struct GameState 
{
    pub zones: HashMap<Zone, Vec<Card>>,

    pub lands: u32,
    pub life: i32,
    pub turns: u32,

    pub step: GameStep,
}

impl GameState 
{
    // TODO: Pass through list of players and their chosen decks instead of just one deck
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

        GameState 
        {
            zones,
            lands: 0,
            life: 20,
            turns: 0,
            step: GameStep::StartTurn,
        }
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
                self.step = GameStep::Draw;
            }

            GameStep::Draw =>
            {
                let card = 
                {
                    let library = self.zones.get_mut(&Zone::Library).unwrap();
                    library.pop()
                };

                if let Some(card) = card 
                {
                    let hand = self.zones.get_mut(&Zone::Hand).unwrap();
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
                        let hand = self.zones.get_mut(&Zone::Hand).unwrap();
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
                        self.lands += 1;
                        let battlefield = self.zones.get_mut(&Zone::Battlefield).unwrap();
                        battlefield.push(card);
                    }
                }

                // Cast creatures
                {
                    let mut i = 0;
                    loop
                    {
                        let hand_len = self.zones.get(&Zone::Hand).unwrap().len();
                        if i >= hand_len
                        {
                            break;
                        }

                        let castable;
                        {
                            let hand = self.zones.get(&Zone::Hand).unwrap();
                            castable = crate::creature::is_creature(&hand[i]) && hand[i].cost <= self.lands;
                        }

                        if castable
                        {
                            // Remove card first
                            let card = 
                            {
                                let hand = self.zones.get_mut(&Zone::Hand).unwrap();
                                hand.remove(i)
                            };

                            self.lands -= card.cost; // adjust mana
                            vlog!(ELoggingVerbosity::Verbose, "Cast {}", card.name);

                            let battlefield = self.zones.get_mut(&Zone::Battlefield).unwrap();
                            battlefield.push(card);
                        }
                        else
                        {
                            i += 1;
                        }
                    }
                }

                self.step = GameStep::Combat;
            }

            GameStep::Combat =>
            {
                let battlefield = self.zones.get(&Zone::Battlefield).unwrap();
                let mut damage = 0;
                for card in battlefield.iter()
                {
                    damage += crate::creature::creature_stats(card).map(|s| s.power as u32).unwrap_or(0);
                }

                self.life -= damage as i32;

                if self.life <= 0
                {
                    self.step = GameStep::GameOver;
                }
                else
                {
                    self.step = GameStep::EndTurn;
                }
            }

            GameStep::EndTurn =>
            {
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
        println!("Life: {}", self.life);

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
            let cards = self.zones.get(zone).unwrap();
            println!("{:?}: {} cards", zone, cards.len());
        }
    }

    pub fn describe_verbose(&self)
    {
        for zone in &[Zone::Hand, Zone::Battlefield, Zone::Library, Zone::Graveyard]
        {
            let cards = self.zones.get(zone).unwrap();
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
                    let mut card_groups: HashMap<&str, u32> = HashMap::new();
                    for card in cards.iter()
                    {
                        *card_groups.entry(card.name).or_insert(0) += 1;
                    }

                    for (name, count) in card_groups.iter()
                    {
                        println!("  {} x{}", name, count);
                    }
                }
                Zone::Hand =>
                {
                    // Print hand cards grouped by count in an inline list
                    let mut groups: HashMap<&str, u32> = HashMap::new();
                    for card in cards.iter()
                    {
                        *groups.entry(card.name).or_insert(0) += 1;
                    }

                    let mut items: Vec<(&str, u32)> = groups.into_iter().collect();
                    items.sort_by(|a, b| a.0.cmp(b.0));

                    let mut parts: Vec<String> = Vec::new();
                    for (name, count) in items.iter()
                    {
                        if *count > 1
                        {
                            parts.push(format!("{} x{}", name, count));
                        }
                        else
                        {
                            parts.push(name.to_string());
                        }
                    }

                    if !parts.is_empty()
                    {
                        println!("  {}", parts.join(", "));
                    }
                }
                Zone::Battlefield =>
                {
                    // Group identical cards together with counts
                    let mut card_groups: HashMap<&str, (u8, u8, bool, u32)> = HashMap::new();
                    for card in cards.iter()
                    {
                        let power = crate::creature::creature_stats(card).map(|s| s.power).unwrap_or(0);
                        let toughness = crate::creature::creature_stats(card).map(|s| s.toughness).unwrap_or(0);
                        let is_creature = crate::creature::is_creature(card);
                        card_groups.entry(card.name)
                            .and_modify(|(_, _, _, count)| *count += 1)
                            .or_insert((power, toughness, is_creature, 1));
                    }

                    for (name, (power, toughness, is_creature, count)) in card_groups.iter()
                    {
                        if *is_creature
                        {
                            if *count > 1
                            {
                                println!("  {}: {}/{} x{}", name, power, toughness, count);
                            }
                            else
                            {
                                println!("  {}: {}/{}", name, power, toughness);
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
