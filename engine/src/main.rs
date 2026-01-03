use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::any::Any;
use std::io::{self, Write};

#[repr(u8)]
#[derive(Debug, Copy, Eq, Ord, Clone, PartialEq, PartialOrd)]
pub enum ELoggingVerbosity 
{
    Error = 0,
    Warning = 1,
    Normal = 2,
    Verbose = 3,
    VeryVerbose = 4,
}

// TODO: rename to EGamePhase
// TODO: split out into a ETurnPhase and a EGamePhase
#[derive(Copy, Clone, Debug, PartialEq)]
enum GameStep 
{
    StartTurn,
    Draw,
    Main,
    Combat,
    EndTurn,
    GameOver,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Zone
{
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum StepCommand
{
    StepPhase,       // "s"
    StepTurn,        // "t"
    RunGame,         // "g"
    RunDeck,         // "d"
    RunAll,          // "r"
    Quit,            // "q"
    Invalid,         // anything else
}

fn parse_command(input: &str) -> StepCommand
{
    match input
    {
        "s" => StepCommand::StepPhase,
        "t" => StepCommand::StepTurn,
        "g" => StepCommand::RunGame,
        "d" => StepCommand::RunDeck,
        "r" => StepCommand::RunAll,
        "q" => StepCommand::Quit,
        _   => StepCommand::Invalid,
    }
}

static GLOBAL_VERBOSITY: AtomicUsize = AtomicUsize::new(ELoggingVerbosity::Normal as usize);

pub fn set_global_verbosity(level: ELoggingVerbosity) 
{
    GLOBAL_VERBOSITY.store(level as usize, Ordering::Relaxed);
}

pub fn global_verbosity() -> ELoggingVerbosity 
{
    match GLOBAL_VERBOSITY.load(Ordering::Relaxed) 
    {
        0 => ELoggingVerbosity::Error,
        1 => ELoggingVerbosity::Warning,
        2 => ELoggingVerbosity::Normal,
        3 => ELoggingVerbosity::Verbose,
        _ => ELoggingVerbosity::VeryVerbose,
    }
}

#[macro_export]
macro_rules! vlog
{
    ($level:expr, $fmt:expr $(, $args:expr)* $(,)?) => 
    {{
        if ($level as usize) <= crate::global_verbosity() as usize
        {
            println!($fmt $(, $args)*);
        }
    }};
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum CardType 
{
    Land,
    Creature,
}

// Use composition so only creatures have power/toughness.
#[derive(Copy, Clone, Debug)]
struct CreatureStats
{
    power: u8,
    toughness: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum CardFragmentKind
{
    Creature,
}

trait Fragment: Any + Send + Sync
{
    fn as_any(&self) -> &dyn Any;
    fn kind(&self) -> CardFragmentKind;
    fn box_clone(&self) -> Box<dyn Fragment>;
}

#[derive(Clone, Debug)]
struct CreatureFragment
{
    stats: CreatureStats,
}

impl Fragment for CreatureFragment
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    fn kind(&self) -> CardFragmentKind
    {
        CardFragmentKind::Creature
    }
    fn box_clone(&self) -> Box<dyn Fragment>
    {
        Box::new(CreatureFragment { stats: self.stats })
    }
}

impl Clone for Box<dyn Fragment>
{
    fn clone(&self) -> Box<dyn Fragment>
    {
        self.box_clone()
    }
}

#[derive(Clone)]
struct Card
{
    name: &'static str,
    card_types: Vec<CardType>,
    cost: u32,
    fragments: HashMap<CardFragmentKind, Box<dyn Fragment>>,
}

impl Card
{
    fn is_type(&self, t: CardType) -> bool
    {
        self.card_types.iter().any(|ct| *ct == t)
    }
    fn add_type(&mut self, t: CardType)
    {
        if !self.card_types.contains(&t)
        {
            self.card_types.push(t);
        }
    }

    fn remove_type(&mut self, t: CardType)
    {
        if let Some(pos) = self.card_types.iter().position(|ct| *ct == t)
        {
            self.card_types.remove(pos);
        }
    }
}

mod creature
{
    use super::{Card, CardType, CardFragmentKind, CreatureFragment, CreatureStats};

    pub fn is_creature(card: &Card) -> bool
    {
        card.card_types.iter().any(|ct| *ct == CardType::Creature)
            || card.fragments.contains_key(&CardFragmentKind::Creature)
    }

    pub fn creature_stats(card: &Card) -> Option<CreatureStats>
    {
        card.fragments.get(&CardFragmentKind::Creature).and_then(|f|
            f.as_any().downcast_ref::<CreatureFragment>().map(|cf| cf.stats)
        )
    }

    pub fn add_creature_fragment(card: &mut Card, power: u8, toughness: u8)
    {
        card.fragments.insert(
            CardFragmentKind::Creature,
            Box::new(CreatureFragment { stats: CreatureStats { power, toughness } }),
        );
    }

    pub fn remove_creature_fragment(card: &mut Card)
    {
        card.fragments.remove(&CardFragmentKind::Creature);
    }
}

#[derive(Clone)]
struct Deck
{
    cards: Vec<Card>,
}

impl Deck
{
    fn count(&self, card_type: CardType) -> usize 
    {
        self.cards.iter().filter(|c| c.is_type(card_type)).count()
    }
}

fn forest() -> Card 
{
    Card
    {
        name: "Forest",
        card_types: vec![CardType::Land],
        cost: 0,
        fragments: HashMap::new(),
    }
}

fn grizzly_bears() -> Card 
{
    Card
    {
        name: "Grizzly Bears",
        card_types: vec![CardType::Creature],
        cost: 2,
        fragments: {
            let mut m = HashMap::new();
            m.insert(
                CardFragmentKind::Creature,
                Box::new(CreatureFragment { stats: CreatureStats { power: 2, toughness: 2 } }) as Box<dyn Fragment>,
            );
            m
        },
    }
}

struct ProgramState 
{
    step_mode: StepCommand,
}

impl ProgramState
{
    fn new() -> Self
    {
        ProgramState
        {
            step_mode: StepCommand::StepPhase,
        }
    }
}

struct GameState 
{
    zones: HashMap<Zone, Vec<Card>>,

    lands: u32,
    life: i32,
    turns: u32,

    step: GameStep,
}

impl GameState 
{
    // TODO: Pass through list of players and their chosen decks instead of just one deck
    fn new(deck: &Deck) -> Self 
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
    fn step(&mut self)
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
                        if let Some(pos) = hand.iter().position(|c| c.is_type(CardType::Land))
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

    fn is_game_over(&self) -> bool
    {
        self.step == GameStep::GameOver
    }

    fn describe(&self, verbose: bool)
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

    fn describe_summary(&self)
    {
        // Print only zone counts
        for zone in &[Zone::Hand, Zone::Battlefield, Zone::Library, Zone::Graveyard, Zone::Exile]
        {
            let cards = self.zones.get(zone).unwrap();
            println!("{:?}: {} cards", zone, cards.len());
        }
    }

    fn describe_verbose(&self)
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

fn wait_for_command() -> StepCommand
{
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    parse_command(input.trim())
}

fn simulate_game(deck: &Deck, step_mode: StepCommand) -> (u32, StepCommand)
{
    let mut game = GameState::new(deck);
    let mut mode = step_mode;

    loop
    {
        match mode
        {
            StepCommand::StepPhase =>
            {
                if game.is_game_over()
                {
                    break;
                }

                game.step();
                game.describe(true);

                // get new command
                mode = wait_for_command();
            }

            StepCommand::StepTurn =>
            {
                // Step one whole turn (StartTurn -> EndTurn)
                if game.is_game_over()
                {
                    break;
                }

                loop
                {
                    game.step();
                    if game.step == GameStep::StartTurn || game.is_game_over()
                    {
                        break;
                    }
                }

                game.describe(true);
                mode = wait_for_command();
            }

            StepCommand::RunGame | StepCommand::RunDeck | StepCommand::RunAll =>
            {
                while !game.is_game_over()
                {
                    game.step();
                }

                if mode == StepCommand::RunGame
                {
                    game.describe(true);
                    println!("Game over in {} turns.", game.turns);

                    // get next command
                    mode = wait_for_command();
                }

                // exit after running to completion
                break;
            }

            StepCommand::Quit =>
            {
                break;
            }

            StepCommand::Invalid =>
            {
                mode = wait_for_command();
            }
        }
    }

    (game.turns, mode)
}

fn try_scenario(lands: u32, nonlands: u32, program_state: &mut ProgramState) -> f64
{
    let mut cards = Vec::new();

    for _ in 0..lands
    {
        cards.push(forest());
    }

    for _ in 0..nonlands
    {
        cards.push(grizzly_bears());
    }

    let deck = Deck { cards };
    let games = 10000;
    let mut total_turns = 0;

    for _ in 0..games
    {
        let (turns, new_mode) = simulate_game(&deck, program_state.step_mode);
        total_turns += turns;

        // update ProgramState after simulate_game
        program_state.step_mode = new_mode;
    }

    let avg_turns_to_death = total_turns as f64 / games as f64;

    if program_state.step_mode != StepCommand::Quit
    {
        println!(
            "Average turns to death for deck with {} lands and {} nonlands over {} games: {:.4}",
            lands,
            nonlands,
            games,
            avg_turns_to_death
        );
    }

    avg_turns_to_death
}

fn main()
{
    set_global_verbosity(ELoggingVerbosity::Normal);

    let mut program_state = ProgramState::new();

    println!("TCG Simulator");
    println!("Commands:");
    println!("  s  -> step one phase");
    println!("  t  -> step one whole turn");
    println!("  g  -> run the current game to completion");
    println!("  d  -> run the simulation to completion for the current deck");
    println!("  r  -> run the whole simulation to completion (all decks)");
    println!("  q  -> quit");
    println!();

    let land_count = 29;
    let nonland_count = 31;
    let change_size = 1;

    program_state.step_mode = wait_for_command();

    let result0 = try_scenario(land_count, nonland_count, &mut program_state);
    if program_state.step_mode == StepCommand::RunDeck
    {
        program_state.step_mode = wait_for_command();
    }

    let mut result1 = 0.0;

    if program_state.step_mode != StepCommand::Quit
    {
        result1 = try_scenario(land_count + change_size, nonland_count - change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = wait_for_command();
        }
    }

    let mut result2 = 0.0;
    if program_state.step_mode != StepCommand::Quit
    {   
        result2 = try_scenario(land_count - change_size, nonland_count + change_size, &mut program_state);
    }

    if program_state.step_mode != StepCommand::Quit
    {
         let smallest_turns_to_death = result0.min(result1).min(result2);

        if result0 == smallest_turns_to_death
        {
            vlog!(ELoggingVerbosity::Normal, "Suggestion: Deck is decent as-is");
        }
        else if result1 == smallest_turns_to_death
        {
            vlog!(ELoggingVerbosity::Normal, "Suggestion: Try more land cards.");
        }
        else if result2 == smallest_turns_to_death
        {
            vlog!(ELoggingVerbosity::Normal, "Suggestion: Try more nonland cards.");
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn card_composition_and_type_mutation()
    {
        let f = forest();
        assert!(!crate::creature::is_creature(&f));
        assert!(crate::creature::creature_stats(&f).is_none());

        let mut g = grizzly_bears();
        assert!(crate::creature::is_creature(&g));
        assert!(crate::creature::creature_stats(&g).is_some());
        assert_eq!(crate::creature::creature_stats(&g).unwrap().power, 2);

        // remove creature type (doesn't automatically remove fragment)
        g.remove_type(CardType::Creature);
        assert!(!g.is_type(CardType::Creature));

        // fragment still present until explicitly removed
        crate::creature::remove_creature_fragment(&mut g);
        assert!(!crate::creature::is_creature(&g));
        assert!(crate::creature::creature_stats(&g).is_none());

        // add creature type back and set creature fragment
        g.add_type(CardType::Creature);
        crate::creature::add_creature_fragment(&mut g, 3, 3);
        assert!(crate::creature::is_creature(&g));
        assert_eq!(crate::creature::creature_stats(&g).unwrap().power, 3);
    }
}
