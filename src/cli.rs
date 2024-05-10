use crate::data::Backend;
use crate::effects::StackEffect;
use crate::output::{NoiseLevel, OutputFormat};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::str::FromStr;
use std::{error, fmt};

mod interact;
use interact::*;

/// The current version of the CLI. (As defined in Cargo.toml)
pub const SIGI_VERSION: &str = std::env!("CARGO_PKG_VERSION");

const DEFAULT_STACK_NAME: &str = "sigi";
const DEFAULT_FORMAT: OutputFormat = OutputFormat::Human(NoiseLevel::Normal);
const DEFAULT_BACKEND: Backend = Backend::HomeDir;
const DEFAULT_SHORT_LIST_LIMIT: usize = 10;

// === Glossary ===
const COMPLETE_TERMS: [&str; 4] = ["complete", "done", "finish", "fulfill"];
const COUNT_TERMS: [&str; 3] = ["count", "size", "length"];
const DELETE_TERMS: [&str; 5] = ["delete", "pop", "remove", "cancel", "drop"];
const DELETE_ALL_TERMS: [&str; 6] = [
    "delete-all",
    "purge",
    "pop-all",
    "remove-all",
    "cancel-all",
    "drop-all",
];
const HEAD_TERMS: [&str; 3] = ["head", "top", "first"];
const IS_EMPTY_TERMS: [&str; 2] = ["is-empty", "empty"];
const LIST_TERMS: [&str; 4] = ["list", "ls", "snoop", "all"];
const LIST_STACKS_TERMS: [&str; 2] = ["list-stacks", "stacks"];
const MOVE_TERMS: [&str; 1] = ["move"];
const MOVE_ALL_TERMS: [&str; 1] = ["move-all"];
const NEXT_TERMS: [&str; 4] = ["next", "later", "cycle", "bury"];
const PEEK_TERMS: [&str; 2] = ["peek", "show"];
const PICK_TERMS: [&str; 1] = ["pick"];
const PUSH_TERMS: [&str; 6] = ["push", "create", "add", "do", "start", "new"];
const ROT_TERMS: [&str; 2] = ["rot", "rotate"];
const SWAP_TERMS: [&str; 1] = ["swap"];
const TAIL_TERMS: [&str; 3] = ["tail", "bottom", "last"];
// === /glossary ===

pub fn run() {
    let args = Cli::parse();

    let stack = args.stack.unwrap_or_else(|| DEFAULT_STACK_NAME.into());

    match args.mode {
        None => {
            let output = args.fc.into_output_format().unwrap_or(DEFAULT_FORMAT);
            let peek = StackEffect::Peek { stack };
            peek.run(&DEFAULT_BACKEND, &output);
        }
        Some(Mode::Command(command)) => {
            let (effect, effect_fc) = command.into_effect_and_fc(stack);
            let output = args.fc.into_fallback_for(effect_fc);
            effect.run(&DEFAULT_BACKEND, &output);
        }
        Some(Mode::Interactive { fc }) => {
            let output = args.fc.into_fallback_for(fc);
            interact(stack, output);
        }
        Some(Mode::ReadStdin) => interact(stack, OutputFormat::TerseText),
    };
}

#[derive(Parser)]
#[command(name = "sigi", version = SIGI_VERSION, after_help = INTERACT_INSTRUCTIONS, after_long_help = INTERACT_LONG_INSTRUCTIONS)]
/// An organizing tool for terminal lovers who hate organizing
struct Cli {
    #[command(flatten)]
    fc: FormatConfig,

    #[arg(short='t', long, visible_aliases = &["topic", "about", "namespace"])]
    /// Manage items in a specific stack
    stack: Option<String>,

    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand)]
enum Mode {
    /// Run in an interactive mode
    #[command(visible_alias = "i")]
    Interactive {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Read input lines from standard input. Same commands as interactive
    /// mode, but only prints for printing commands. Intended for use in unix
    /// pipes
    #[command(name = "-")]
    ReadStdin,

    #[command(flatten)]
    Command(Command),
}

#[derive(Subcommand)]
enum Command {
    /// Move the current item to "<STACK>_history" and mark as completed
    #[command(visible_aliases = &COMPLETE_TERMS[1..])]
    Complete {
        /// The number of the item to complete. Default is the most recent item (0 index)
        n: Option<usize>,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print the total number of items in the stack
    #[command(visible_aliases = &COUNT_TERMS[1..])]
    Count {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Move the current item to "<STACK>_history" and mark as deleted
    #[command(visible_aliases = &DELETE_TERMS[1..])]
    Delete {
        /// The number of the item to delete. Default is the most recent item (0 index)
        n: Option<usize>,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Move all items to "<STACK>_history" and mark as deleted
    #[command(visible_aliases = &DELETE_ALL_TERMS[1..])]
    DeleteAll {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print the first N items (default is 10)
    #[command(visible_aliases = &HEAD_TERMS[1..])]
    Head {
        /// The number of items to display
        n: Option<usize>,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print "true" if stack has zero items, or print "false" (and exit with a
    /// nonzero exit code) if the stack does have items
    #[command(visible_aliases = &IS_EMPTY_TERMS[1..])]
    IsEmpty {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print all items
    #[command(visible_aliases = &LIST_TERMS[1..])]
    List {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print all stacks
    #[command(visible_aliases = &LIST_STACKS_TERMS[1..])]
    ListStacks {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Move current item to another stack
    #[command(arg_required_else_help = true, visible_aliases = &MOVE_TERMS[1..])]
    Move {
        #[arg(name = "destination")]
        /// The stack that will get the source stack's current item
        dest: String,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Move all items to another stack
    #[command(arg_required_else_help = true, visible_aliases = &MOVE_ALL_TERMS[1..])]
    MoveAll {
        #[arg(name = "destination")]
        /// The stack that will get all the source stack's items
        dest: String,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Cycle to the next item; the current item becomes last
    #[command(visible_aliases = &NEXT_TERMS[1..])]
    Next {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print the first item. This is the default CLI behavior when no command is given
    #[command(visible_aliases = &PEEK_TERMS[1..])]
    Peek {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Move items to the top of stack by their number
    #[command(visible_aliases = &PICK_TERMS[1..])]
    Pick {
        ns: Vec<usize>,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Create a new item
    #[command(visible_aliases = &PUSH_TERMS[1..])]
    Push {
        // The content to add as an item. Multiple arguments will be interpreted as a single string
        content: Vec<String>,

        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Rotate the three most-current items
    #[command(visible_aliases = &ROT_TERMS[1..])]
    Rot {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Swap the two most-current items
    #[command(visible_aliases = &SWAP_TERMS[1..])]
    Swap {
        #[command(flatten)]
        fc: FormatConfig,
    },

    /// Print the last N items (default is 10)
    #[command(visible_aliases = &TAIL_TERMS[1..])]
    Tail {
        /// The number of items to display
        n: Option<usize>,

        #[command(flatten)]
        fc: FormatConfig,
    },
}

impl Command {
    fn into_effect_and_fc(self, stack: String) -> (StackEffect, FormatConfig) {
        use StackEffect::*;
        match self {
            Command::Complete { n, fc } => (
                Complete {
                    stack,
                    index: n.unwrap_or(0),
                },
                fc,
            ),
            Command::Count { fc } => (Count { stack }, fc),
            Command::Delete { n, fc } => (
                Delete {
                    stack,
                    index: n.unwrap_or(0),
                },
                fc,
            ),
            Command::DeleteAll { fc } => (DeleteAll { stack }, fc),
            Command::Head { n, fc } => {
                let n = n.unwrap_or(DEFAULT_SHORT_LIST_LIMIT);
                (Head { n, stack }, fc)
            }
            Command::IsEmpty { fc } => (IsEmpty { stack }, fc),
            Command::List { fc } => (ListAll { stack }, fc),
            Command::ListStacks { fc } => (ListStacks, fc),
            Command::Move { dest, fc } => (Move { stack, dest }, fc),
            Command::MoveAll { dest, fc } => (MoveAll { stack, dest }, fc),
            Command::Next { fc } => (Next { stack }, fc),
            Command::Peek { fc } => (Peek { stack }, fc),
            Command::Pick { ns, fc } => (Pick { stack, indices: ns }, fc),
            Command::Push { content, fc } => {
                let content = content.join(" ");
                (Push { stack, content }, fc)
            }
            Command::Rot { fc } => (Rot { stack }, fc),
            Command::Swap { fc } => (Swap { stack }, fc),
            Command::Tail { n, fc } => {
                let n = n.unwrap_or(DEFAULT_SHORT_LIST_LIMIT);
                (Tail { n, stack }, fc)
            }
        }
    }
}

#[derive(Args)]
struct FormatConfig {
    #[arg(short, long)]
    /// Omit any leading labels or symbols. Recommended for use in shell scripts
    quiet: bool,

    #[arg(short, long)]
    /// Omit any output at all
    silent: bool,

    #[arg(short, long, visible_alias = "noisy")]
    /// Print more information, like when an item was created
    verbose: bool,

    #[arg(short, long)]
    /// Use a programmatic format. Options include [csv, json, json-compact, tsv]. Not compatible with quiet/silent/verbose.
    format: Option<ProgrammaticFormat>,
}

impl FormatConfig {
    fn into_output_format(self) -> Option<OutputFormat> {
        let FormatConfig {
            verbose,
            silent,
            quiet,
            format,
        } = self;

        use NoiseLevel::*;
        use OutputFormat::*;

        format
            .map(|format| match format {
                ProgrammaticFormat::Csv => Csv,
                ProgrammaticFormat::Json => Json,
                ProgrammaticFormat::JsonCompact => JsonCompact,
                ProgrammaticFormat::Tsv => Tsv,
            })
            .or(if verbose {
                Some(Human(Verbose))
            } else if silent {
                Some(Silent)
            } else if quiet {
                Some(Human(Quiet))
            } else {
                None
            })
    }

    fn into_fallback_for(self, fc: FormatConfig) -> OutputFormat {
        fc.into_output_format()
            .or_else(|| self.into_output_format())
            .unwrap_or(DEFAULT_FORMAT)
    }
}

#[derive(ValueEnum, Clone)]
enum ProgrammaticFormat {
    Csv,
    Json,
    JsonCompact,
    Tsv,
}

impl FromStr for ProgrammaticFormat {
    type Err = UnknownFormat;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        use ProgrammaticFormat::*;

        let format = format.to_ascii_lowercase();

        match format.as_str() {
            "csv" => Ok(Csv),
            "json" => Ok(Json),
            "json-compact" => Ok(JsonCompact),
            "tsv" => Ok(Tsv),
            _ => Err(UnknownFormat { format }),
        }
    }
}

#[derive(Debug)]
struct UnknownFormat {
    format: String,
}

impl error::Error for UnknownFormat {}

impl fmt::Display for UnknownFormat {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(out, "Unknown format: {}", self.format)
    }
}
