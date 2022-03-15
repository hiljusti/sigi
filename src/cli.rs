use crate::data::Item;
use crate::effects::*;
use crate::output::{NoiseLevel, OutputFormat};
use clap::{ArgEnum, Args, Parser, Subcommand};
use std::str::FromStr;
use std::{error, fmt};

/// The current version of the CLI. (As defined in Cargo.toml)
pub const SIGI_VERSION: &str = std::env!("CARGO_PKG_VERSION");

const DEFAULT_STACK_NAME: &str = "sigi";
const DEFAULT_FORMAT: OutputFormat = OutputFormat::Human(NoiseLevel::Normal);

pub fn run() {
    let args = Cli::parse();

    let stack = args.stack.unwrap_or_else(|| DEFAULT_STACK_NAME.into());

    if args.command.is_none() {
        let fmt = args.fc.resolve().unwrap_or(DEFAULT_FORMAT);
        Peek { stack }.run(fmt);
        return;
    }

    let with_fallback = args.fc.as_fallback();

    let command = args.command.unwrap();
    match command {
        Command::Complete { fc } => {
            Complete { stack }.run(with_fallback(fc));
        }
        Command::Count { fc } => {
            Count { stack }.run(with_fallback(fc));
        }
        Command::Delete { fc } => {
            Delete { stack }.run(with_fallback(fc));
        }
        Command::DeleteAll { fc } => {
            DeleteAll { stack }.run(with_fallback(fc));
        }
        Command::Head { n, fc } => {
            Head { n, stack }.run(with_fallback(fc));
        }
        Command::IsEmpty { fc } => {
            IsEmpty { stack }.run(with_fallback(fc));
        }
        Command::List { fc } => {
            ListAll { stack }.run(with_fallback(fc));
        }
        Command::Move { dest, fc } => {
            Move { stack, dest }.run(with_fallback(fc));
        }
        Command::MoveAll { dest, fc } => {
            MoveAll { stack, dest }.run(with_fallback(fc));
        }
        Command::Next { fc } => {
            Next { stack }.run(with_fallback(fc));
        }
        Command::Peek { fc } => {
            Peek { stack }.run(with_fallback(fc));
        }
        Command::Pick { ns, fc } => {
            Pick { stack, indices: ns }.run(with_fallback(fc));
        }
        Command::Push { content, fc } => {
            let item = Item::new(&content.join(" "));
            Push { stack, item }.run(with_fallback(fc));
        }
        Command::Rot { fc } => {
            Rot { stack }.run(with_fallback(fc));
        }
        Command::Swap { fc } => {
            Swap { stack }.run(with_fallback(fc));
        }
        Command::Tail { n, fc } => {
            Tail { stack, n }.run(with_fallback(fc));
        }
    };
}

#[derive(Parser)]
#[clap(name = "sigi", version = SIGI_VERSION)]
/// An organizing tool for terminal lovers who hate organizing
struct Cli {
    #[clap(flatten)]
    fc: FormatConfig,

    #[clap(short='t', long, visible_aliases = &["topic", "about", "namespace"])]
    /// Manage items in a specific stack
    stack: Option<String>,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Move the current item to "<STACK>_history" and mark as completed
    #[clap(visible_aliases = &["done", "finish", "fulfill"])]
    Complete {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Print the total number of items in the stack
    #[clap(visible_aliases = &["size", "length"])]
    Count {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Move the current item to "<STACK>_history" and mark as deleted
    #[clap(visible_aliases = &["pop", "remove", "cancel", "drop"])]
    Delete {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Move all items to "<STACK>_history" and mark as deleted
    #[clap(visible_aliases = &["purge", "pop-all", "remove-all", "cancel-all", "drop-all"])]
    DeleteAll {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// List the first N items (default is 10)
    #[clap(visible_aliases = &["top", "first"])]
    Head {
        /// The number of items to display
        n: Option<usize>,

        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Print "true" if stack has zero items, or print "false" (and exit with a
    /// nonzero exit code) if the stack does have items
    #[clap(visible_aliases = &["empty"])]
    IsEmpty {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// List all items
    #[clap(visible_aliases = &["ls", "snoop", "show", "all"])]
    List {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Move current item to another stack
    #[clap(arg_required_else_help = true)]
    Move {
        #[clap(name = "destination")]
        /// The stack that will get the source stack's current item
        dest: String,

        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Move all items to another stack
    #[clap(arg_required_else_help = true)]
    MoveAll {
        #[clap(name = "destination")]
        /// The stack that will get all the source stack's items
        dest: String,

        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Cycle to the next item; the current item becomes last
    #[clap(visible_aliases = &["later", "cycle", "bury"])]
    Next {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Show the first item. This is the default behavior when no command is given
    #[clap(visible_aliases = &["show"])]
    Peek {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Move items to the top of stack by their number
    Pick {
        ns: Vec<usize>,

        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Create a new item
    #[clap(visible_aliases = &["create", "add", "do", "start", "new"])]
    Push {
        // The content to add as an item. Multiple arguments will be interpreted as a single string
        content: Vec<String>,

        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Rotate the three most-current items
    #[clap(visible_aliases = &["rotate"])]
    Rot {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// Swap the two most-current items
    Swap {
        #[clap(flatten)]
        fc: FormatConfig,
    },

    /// List the last N items (default is 10)
    #[clap(visible_aliases = &["bottom", "last"])]
    Tail {
        /// The number of items to display
        n: Option<usize>,

        #[clap(flatten)]
        fc: FormatConfig,
    },
}

#[derive(Args)]
struct FormatConfig {
    #[clap(short, long)]
    /// Omit any leading labels or symbols. Recommended for use in shell scripts
    quiet: bool,

    #[clap(short, long)]
    /// Omit any output at all
    silent: bool,

    #[clap(short, long, visible_alias = "noisy")]
    /// Print more information, like when an item was created
    verbose: bool,

    #[clap(short, long)]
    /// Use a programmatic format. Options include [csv, json, json-compact, tsv]. Not compatible with quiet/silent/verbose.
    format: Option<ProgrammaticFormat>,
}

impl FormatConfig {
    fn resolve(self) -> Option<OutputFormat> {
        let FormatConfig {
            verbose,
            silent,
            quiet,
            format,
        } = self;
        format
            .map(|format| match format {
                ProgrammaticFormat::Csv => OutputFormat::Csv,
                ProgrammaticFormat::Json => OutputFormat::Json,
                ProgrammaticFormat::JsonCompact => OutputFormat::JsonCompact,
                ProgrammaticFormat::Tsv => OutputFormat::Tsv,
            })
            .or_else(|| {
                if verbose {
                    Some(OutputFormat::Human(NoiseLevel::Verbose))
                } else if silent {
                    Some(OutputFormat::Silent)
                } else if quiet {
                    Some(OutputFormat::Human(NoiseLevel::Quiet))
                } else {
                    None
                }
            })
    }

    fn as_fallback(self) -> impl FnOnce(FormatConfig) -> OutputFormat {
        |fc: FormatConfig| {
            fc.resolve()
                .or_else(|| self.resolve())
                .unwrap_or(DEFAULT_FORMAT)
        }
    }
}

#[derive(ArgEnum, Clone)]
#[clap(arg_enum)]
enum ProgrammaticFormat {
    Csv,
    Json,
    JsonCompact,
    Tsv,
}

#[derive(Debug)]
struct UnknownFormat {
    format: String,
}

impl fmt::Display for UnknownFormat {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(out, "Unknown format: {}", self.format)
    }
}

impl error::Error for UnknownFormat {}

impl FromStr for ProgrammaticFormat {
    type Err = UnknownFormat;
    fn from_str(format: &str) -> Result<Self, Self::Err> {
        let format = format.to_ascii_lowercase();
        match format.as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            "json-compact" => Ok(Self::JsonCompact),
            "tsv" => Ok(Self::Tsv),
            _ => Err(UnknownFormat { format }),
        }
    }
}
