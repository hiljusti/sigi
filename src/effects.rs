use crate::output::OutputFormat;

pub mod lifecycle;
pub use lifecycle::*;
pub mod views;
pub use views::*;
pub mod shuffle;
pub use shuffle::*;
pub mod housekeeping;
pub use housekeeping::*;

const HISTORY_SUFFIX: &str = "_history";

pub trait StackEffect {
    fn run(&self, output: OutputFormat);
}

// ===== Helper functions =====

fn stack_history_of(stack: &str) -> String {
    stack.to_string() + HISTORY_SUFFIX
}
