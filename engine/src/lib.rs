use std::sync::atomic::{AtomicUsize, Ordering};

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
        if ($level as usize) <= $crate::global_verbosity() as usize
        {
            println!($fmt $(, $args)*);
        }
    }};
}

pub mod card;
pub mod creature;
pub mod game;
pub mod sim;

pub use crate::card::*;
pub use crate::creature::*;
pub use crate::game::*;
pub use crate::sim::*;
