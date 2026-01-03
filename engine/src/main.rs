use engine::{set_global_verbosity, ELoggingVerbosity, game::ProgramState, game::StepCommand, sim};
use engine::vlog;

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

    program_state.step_mode = sim::parse_command(&read_line().trim());

    let result0 = sim::try_scenario(land_count, nonland_count, &mut program_state);
    if program_state.step_mode == StepCommand::RunDeck
    {
        program_state.step_mode = sim::parse_command(&read_line().trim());
    }

    let mut result1 = 0.0;

    if program_state.step_mode != StepCommand::Quit
    {
        result1 = sim::try_scenario(land_count + change_size, nonland_count - change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }
    }

    let mut result2 = 0.0;
    if program_state.step_mode != StepCommand::Quit
    {   
        result2 = sim::try_scenario(land_count - change_size, nonland_count + change_size, &mut program_state);
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

fn read_line() -> String
{
    use std::io::{self, Write};
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}
