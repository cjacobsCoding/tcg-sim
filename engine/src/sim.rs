use std::io::{self, Write};

use crate::game::{GameState, ProgramState, StepCommand, GameStep};
use crate::card::Deck;

pub fn parse_command(input: &str) -> StepCommand
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

fn wait_for_command() -> StepCommand
{
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    parse_command(input.trim())
}

pub fn simulate_game(deck: &Deck, step_mode: StepCommand) -> (u32, StepCommand)
{
    let mut game = GameState::new(2, deck); // Default 2 players
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

pub fn try_scenario(lands: u32, nonlands: u32, program_state: &mut ProgramState) -> f64
{
    let mut cards = Vec::new();

    for _ in 0..lands
    {
        cards.push(crate::card::forest());
    }

    for _ in 0..nonlands
    {
        cards.push(crate::card::grizzly_bears());
    }

    let deck = Deck { cards };
    let games = 3000;
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
