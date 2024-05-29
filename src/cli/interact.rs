use super::*;
use crate::effects::StackEffect;
use crate::output::OutputFormat;
use clap::CommandFactory;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::str::FromStr;

const HUMAN_PROMPT: &str = "🌴 ▶ ";

pub const INTERACT_INSTRUCTIONS: &str = "INTERACTIVE MODE:

Use subcommands in interactive mode directly. \
No OPTIONS (flags) are understood in interactive mode. \
The ; character can be used to separate commands.

The following additional commands are available:
    ?               Show the short version of \"help\"
    clear           Clear the terminal screen
    use             Change to the specified stack [aliases: stack]
    exit            Quit interactive mode [aliases: quit, q]";

pub const INTERACT_LONG_INSTRUCTIONS: &str = "INTERACTIVE MODE:

Use subcommands in interactive mode directly. For example:

    🌴 ▶ push a new thing
    Created: a new thing
    🌴 ▶ peek
    Now: a new thing
    🌴 ▶ delete
    Deleted: a new thing
    Now: nothing
    🌴 ▶ exit
    exit: Buen biåhe!

No OPTIONS (flags) are understood in interactive mode.

The ; character can be used to separate commands.

In interactive mode, the following additional commands are available:
    ?
            Show the short version of \"help\"
    clear   
            Clear the terminal screen
    use
            Change to the specified stack [aliases: stack]
    exit
            Quit interactive mode [aliases: quit, q]";

// TODO: pagination/scrollback?
// TODO: more comprehensive tests
pub fn interact(original_stack: String, output: OutputFormat) {
    print_welcome_msg(output);

    let mut rl = DefaultEditor::new().expect("Unable to create readline.");
    let prompt = if output.is_nonquiet_for_humans() {
        HUMAN_PROMPT
    } else {
        ""
    };

    let mut stack = original_stack;

    loop {
        let line = rl.readline(prompt);

        if let Ok(line) = &line {
            rl.add_history_entry(line).unwrap();
        }

        use InteractAction::*;
        let line = line.map_err(handle_error).map(handle_line(&stack));
        let actions = match line {
            Ok(actions) => actions,
            Err(err_action) => vec![err_action],
        };

        for action in actions {
            match action {
                ShortHelp => Cli::command().print_help().unwrap(),
                LongHelp => Cli::command().print_long_help().unwrap(),
                Clear => clearscreen::clear().expect("Failed to clear screen"),
                DoEffect(effect) => effect.run(&DEFAULT_BACKEND, &output),
                UseStack(new_stack) => {
                    stack = new_stack;
                    output.log(vec!["update", "stack"], vec![vec!["Active stack", &stack]]);
                }
                NoContent => (),
                Exit(reason) => {
                    print_goodbye_msg(&reason, output);
                    return;
                }
                MissingArgument(msg) => {
                    output.log(
                        vec!["argument", "error"],
                        vec![vec![&msg, "missing argument"]],
                    );
                }
                Error(msg) => {
                    output.log(
                        vec!["exit-message", "exit-reason"],
                        vec![vec!["Error"], vec![&msg]],
                    );
                    return;
                }
                Unknown(term) => {
                    if output.is_nonquiet_for_humans() {
                        println!("Oops, I don't know {:?}", term);
                    } else {
                        output.log(vec!["term", "error"], vec![vec![&term, "unknown term"]]);
                    };
                }
            };
        }
    }
}

fn print_welcome_msg(output: OutputFormat) {
    if output.is_nonquiet_for_humans() {
        println!("sigi {}", SIGI_VERSION);
        println!(
            "Type \"exit\", \"quit\", or \"q\" to quit. (On Unixy systems, Ctrl+C or Ctrl+D also work)"
        );
        println!("Type \"?\" for quick help, or \"help\" for a more verbose help message.");
        println!();
    }
}

fn print_goodbye_msg(reason: &str, output: OutputFormat) {
    output.log(
        vec!["exit-reason", "exit-message"],
        vec![vec![reason, "Buen biåhe!"]],
    );
}

enum InteractAction {
    ShortHelp,
    LongHelp,
    Clear,
    DoEffect(StackEffect),
    UseStack(String),
    NoContent,
    Exit(String),
    MissingArgument(String),
    Error(String),
    Unknown(String),
}

fn handle_error(err: ReadlineError) -> InteractAction {
    match err {
        ReadlineError::Interrupted => InteractAction::Exit("Ctrl+c".to_string()),
        ReadlineError::Eof => InteractAction::Exit("Ctrl+d".to_string()),
        err => InteractAction::Error(format!("{:?}", err)),
    }
}

fn handle_line(stack: &str) -> impl Fn(String) -> Vec<InteractAction> + '_ {
    |line| {
        line.split(';')
            .map(|s| s.to_string())
            .map(|line| parse_line(line, stack.to_string()))
            .collect()
    }
}

fn parse_line(line: String, stack: String) -> InteractAction {
    let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

    if tokens.is_empty() {
        return InteractAction::NoContent;
    }

    let term = tokens.first().unwrap().to_ascii_lowercase();

    match term.as_str() {
        "?" => InteractAction::ShortHelp,
        "help" => InteractAction::LongHelp,
        "clear" => InteractAction::Clear,
        "exit" | "quit" | "q" => InteractAction::Exit(term),
        "use" | "stack" => match tokens.get(1) {
            Some(stack) => InteractAction::UseStack(stack.to_string()),
            None => InteractAction::MissingArgument("stack name".to_string()),
        },
        _ => match parse_effect(tokens, stack) {
            ParseEffectResult::Effect(effect) => InteractAction::DoEffect(effect),
            ParseEffectResult::NotEffect(parse_res) => parse_res,
            ParseEffectResult::Unknown => InteractAction::Unknown(term),
        },
    }
}

enum ParseEffectResult {
    Effect(StackEffect),
    NotEffect(InteractAction),
    Unknown,
}

fn parse_effect(tokens: Vec<&str>, stack: String) -> ParseEffectResult {
    let term = tokens.first().unwrap_or(&"");

    let parse_n = || tokens.get(1).and_then(|&s| usize::from_str(s).ok());

    use ParseEffectResult::*;
    use StackEffect::*;

    if COMPLETE_TERMS.contains(term) {
        let index = parse_n().unwrap_or(0);
        return Effect(Complete { stack, index });
    }
    if COUNT_TERMS.contains(term) {
        return Effect(Count { stack });
    }
    if DELETE_TERMS.contains(term) {
        let index = parse_n().unwrap_or(0);
        return Effect(Delete { stack, index });
    }
    if DELETE_ALL_TERMS.contains(term) {
        return Effect(DeleteAll { stack });
    }
    if EDIT_TERMS.contains(term) {
        let index = parse_n().unwrap_or(0);
        return Effect(Edit {
            stack,
            editor: resolve_editor(None),
            index,
        });
    }
    if HEAD_TERMS.contains(term) {
        let n = parse_n().unwrap_or(DEFAULT_SHORT_LIST_LIMIT);
        return Effect(Head { stack, n });
    }
    if IS_EMPTY_TERMS.contains(term) {
        return Effect(IsEmpty { stack });
    }
    if LIST_TERMS.contains(term) {
        return Effect(ListAll { stack });
    }
    if LIST_STACKS_TERMS.contains(term) {
        return Effect(ListStacks);
    }
    if MOVE_TERMS.contains(term) {
        match tokens.get(1) {
            Some(dest) => {
                let dest = dest.to_string();
                return Effect(Move { stack, dest });
            }
            None => {
                return NotEffect(InteractAction::MissingArgument(
                    "destination stack".to_string(),
                ));
            }
        };
    }
    if MOVE_ALL_TERMS.contains(term) {
        match tokens.get(1) {
            Some(dest) => {
                let dest = dest.to_string();
                return Effect(MoveAll { stack, dest });
            }
            None => {
                return NotEffect(InteractAction::MissingArgument(
                    "destination stack".to_string(),
                ));
            }
        };
    }
    if NEXT_TERMS.contains(term) {
        return Effect(Next { stack });
    }
    if PEEK_TERMS.contains(term) {
        return Effect(Peek { stack });
    }
    if PICK_TERMS.contains(term) {
        let indices = tokens
            .iter()
            .filter_map(|s| usize::from_str(s).ok())
            .collect();
        return Effect(Pick { stack, indices });
    }
    if PUSH_TERMS.contains(term) {
        // FIXME: This is convenient, but normalizes whitespace. (E.g. multiple spaces always collapsed, tabs to spaces, etc)
        let content = tokens[1..].join(" ");
        return Effect(Push { stack, content });
    }
    if ROT_TERMS.contains(term) {
        return Effect(Rot { stack });
    }
    if SWAP_TERMS.contains(term) {
        return Effect(Swap { stack });
    }
    if TAIL_TERMS.contains(term) {
        let n = parse_n().unwrap_or(DEFAULT_SHORT_LIST_LIMIT);
        return Effect(Tail { stack, n });
    }

    Unknown
}
