use engine::{set_global_verbosity, ELoggingVerbosity, game::ProgramState, game::StepCommand, sim, music::{MusicPlayer, MusicConfig, music_dir_path}};
use engine::vlog;
use std::collections::HashMap;

fn main()
{
    set_global_verbosity(ELoggingVerbosity::Normal);

    // Initialize background music
    let music_config = MusicConfig {
        fade_duration_ms: 1500,      // 1.5 second fade between songs
        delay_between_songs_ms: 2000, // 2 second delay between songs
        volume: 0.3,                  // 30% volume
    };
    let music_path = music_dir_path();
    let _music_player = MusicPlayer::new(music_path.to_str().unwrap_or("web/music"), music_config);
    _music_player.start();

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

    let mut current_lands = 28;
    let mut current_nonlands = 32;
    let change_size = 1;

    program_state.step_mode = sim::parse_command(&read_line().trim());

    // Hill-climbing algorithm: track results and find consensus among 3+ runs
    let mut result_history: HashMap<(u32, u32), Vec<f64>> = HashMap::new();
    let mut iteration = 1;

    let mut win_counts: HashMap<(u32, u32), u32> = HashMap::new();

    loop
    {
        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        println!("\n=== Iteration {} ===", iteration);
        println!("Testing land/nonland ratios centered around {} lands, {} nonlands", current_lands, current_nonlands);

        // Test three configurations: current, +1 lands, -1 lands
        let result0 = sim::try_scenario(current_lands, current_nonlands, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        let result1 = sim::try_scenario(current_lands + change_size, current_nonlands - change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        let result2 = sim::try_scenario(current_lands - change_size, current_nonlands + change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        // Track results
        result_history.entry((current_lands, current_nonlands)).or_insert_with(Vec::new).push(result0);
        result_history.entry((current_lands + change_size, current_nonlands - change_size)).or_insert_with(Vec::new).push(result1);
        result_history.entry((current_lands - change_size, current_nonlands + change_size)).or_insert_with(Vec::new).push(result2);

        // Determine which configuration was best
        let smallest_turns_to_death = result0.min(result1).min(result2);

        let (best_config_name, best_lands, best_nonlands) = if result0 == smallest_turns_to_death
        {
            ("Current ratio (no change)", current_lands, current_nonlands)
        }
        else if result1 == smallest_turns_to_death
        {
            ("More lands", current_lands + change_size, current_nonlands - change_size)
        }
        else
        {
            ("More nonlands", current_lands - change_size, current_nonlands + change_size)
        };

        let winner_key = (best_lands, best_nonlands);
        let wins = win_counts.entry(winner_key).or_insert(0);
        *wins += 1;

        println!("\nIteration {} Results:", iteration);
        println!("  Current:     {} lands, {} nonlands -> {} avg turns", current_lands, current_nonlands, result0);
        println!("  More lands:  {} lands, {} nonlands -> {} avg turns", current_lands + change_size, current_nonlands - change_size, result1);
        println!("  More nonlands: {} lands, {} nonlands -> {} avg turns", current_lands - change_size, current_nonlands + change_size, result2);
        println!("\nBest configuration: {} ({} lands, {} nonlands) -> {} avg turns (total wins: {})",
            best_config_name, best_lands, best_nonlands, smallest_turns_to_death, *wins);

        // Find decks that have reached 3 wins
        let winners: Vec<_> = win_counts
            .iter()
            .filter(|(_, count)| **count >= 3)
            .map(|(&(l, nl), _)| (l, nl))
            .collect();

        if winners.is_empty() 
        {
            // Continue hill-climbing
            current_lands = best_lands;
            current_nonlands = best_nonlands;
        } 
        else if winners.len() == 1 
        {
            // Clear winner
            let (l, nl) = winners[0];
            println!("\n=== Optimization Complete ===");
            vlog!(
                ELoggingVerbosity::Normal,
                "Final suggestion: {} lands, {} nonlands (3 wins)",
                l,
                nl
            );
            break;
        } 
        else 
        {
            // Multiple decks reached 3 wins simultaneously → tiebreaker
            println!("\nTiebreaker needed between {} decks!", winners.len());

            let mut tiebreaker_results = Vec::new();

            for (l, nl) in winners 
            {
                let r = sim::try_scenario(l, nl, &mut program_state);
                tiebreaker_results.push((l, nl, r));
            }

            let winner = tiebreaker_results
                .iter()
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                .unwrap();

            println!(
                "\nTiebreaker winner: {} lands, {} nonlands -> {:.4}",
                winner.0, winner.1, winner.2);

            vlog!(
                ELoggingVerbosity::Normal,
                "Final suggestion: {} lands, {} nonlands",
                winner.0,
                winner.1
            );
            break;
        }

        iteration += 1;
    }
}

fn read_line() -> String
{
    use std::io::{self, Write};
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}
