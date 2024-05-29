use std::process::Command;

use chrono::Local;

use crate::data::{Backend, Item};
use crate::output::OutputFormat;

const HISTORY_SUFFIX: &str = "_history";

// TODO: Consider more shuffle words: https://docs.factorcode.org/content/article-shuffle-words.html

pub enum StackEffect {
    Push {
        stack: String,
        content: String,
    },
    Complete {
        stack: String,
        index: usize,
    },
    Delete {
        stack: String,
        index: usize,
    },
    DeleteAll {
        stack: String,
    },
    Edit {
        stack: String,
        editor: String,
        index: usize,
    },
    Pick {
        stack: String,
        indices: Vec<usize>,
    },
    Move {
        stack: String,
        dest: String,
    },
    MoveAll {
        stack: String,
        dest: String,
    },
    Swap {
        stack: String,
    },
    Rot {
        stack: String,
    },
    Next {
        stack: String,
    },
    Peek {
        stack: String,
    },
    ListAll {
        stack: String,
    },
    ListStacks,
    Head {
        stack: String,
        n: usize,
    },
    Tail {
        stack: String,
        n: usize,
    },
    Count {
        stack: String,
    },
    IsEmpty {
        stack: String,
    },
}

impl StackEffect {
    pub fn run(self, backend: &Backend, output: &OutputFormat) {
        use StackEffect::*;
        match self {
            Push { stack, content } => push_content(stack, content, backend, output),
            Complete { stack, index } => complete_item(stack, index, backend, output),
            Delete { stack, index } => delete_latest_item(stack, index, backend, output),
            DeleteAll { stack } => delete_all_items(stack, backend, output),
            Edit {
                stack,
                editor,
                index,
            } => edit_item(stack, editor, index, backend, output),
            Pick { stack, indices } => pick_indices(stack, indices, backend, output),
            Move { stack, dest } => move_latest_item(stack, dest, backend, output),
            MoveAll { stack, dest } => move_all_items(stack, dest, backend, output),
            Swap { stack } => swap_latest_two_items(stack, backend, output),
            Rot { stack } => rotate_latest_three_items(stack, backend, output),
            Next { stack } => next_to_latest(stack, backend, output),
            Peek { stack } => peek_latest_item(stack, backend, output),
            ListAll { stack } => list_all_items(stack, backend, output),
            ListStacks => list_stacks(backend, output),
            Head { stack, n } => list_n_latest_items(stack, n, backend, output),
            Tail { stack, n } => list_n_oldest_items(stack, n, backend, output),
            Count { stack } => count_all_items(stack, backend, output),
            IsEmpty { stack } => is_empty(stack, backend, output),
        }
    }
}

fn push_content(stack: String, content: String, backend: &Backend, output: &OutputFormat) {
    let item = Item::new(&content);
    push_item(stack, item, backend, output);
}

fn push_item(stack: String, item: Item, backend: &Backend, output: &OutputFormat) {
    let contents = item.contents.clone();

    let items = if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        items.push(item);
        items
    } else {
        vec![item]
    };

    backend.save(&stack, items).unwrap();

    output.log(vec!["action", "item"], vec![vec!["Created", &contents]]);
}

fn complete_item(stack: String, index: usize, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;

        if items.len() > index {
            let mut item = items.remove(items.len() - index - 1);
            item.mark_completed();

            // Push the now-marked-completed item to history stack.
            push_item(
                stack_history_of(&stack),
                item.clone(),
                backend,
                &OutputFormat::Silent,
            );

            // Save the original stack without that item.
            backend.save(&stack, items).unwrap();

            output.log(
                vec!["action", "item"],
                vec![vec!["Completed", &item.contents]],
            );
        }
    }

    if output.is_nonquiet_for_humans() {
        peek_latest_item(stack, backend, output);
    }
}

fn delete_latest_item(stack: String, index: usize, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;

        if items.len() > index {
            let mut item = items.remove(items.len() - index - 1);
            item.mark_deleted();

            // Push the now-marked-deleted item to history stack.
            push_item(
                stack_history_of(&stack),
                item.clone(),
                backend,
                &OutputFormat::Silent,
            );

            // Save the original stack without that item.
            backend.save(&stack, items).unwrap();

            output.log(
                vec!["action", "item"],
                vec![vec!["Deleted", &item.contents]],
            );
        }
    }

    if output.is_nonquiet_for_humans() {
        peek_latest_item(stack, backend, output);
    }
}

fn delete_all_items(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        items.iter_mut().for_each(|item| item.mark_deleted());
        let n_deleted = items.len();

        // Push the now-marked-deleted items to history stack.
        let history_stack = &stack_history_of(&stack);
        let mut history = backend.load(history_stack).unwrap_or_default();
        history.append(&mut items);
        backend.save(history_stack, history).unwrap();

        // Save the original stack as empty now.
        backend.save(&stack, vec![]).unwrap();

        output.log(
            vec!["action", "item"],
            vec![vec!["Deleted", &format!("{} items", n_deleted)]],
        );
    }
}

fn edit_item(
    stack: String,
    editor: String,
    index: usize,
    backend: &Backend,
    output: &OutputFormat,
) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        if index < items.len() {
            let tmp = std::env::temp_dir().as_path().join("sigi");
            std::fs::create_dir_all(&tmp).unwrap_or_else(|err| {
                panic!(
                    "Unable to create temporary directory {:?} for editing: {}",
                    tmp, err
                )
            });
            let tmpfile = tmp.as_path().join(Local::now().timestamp().to_string());
            std::fs::write(&tmpfile, &items[index].contents).unwrap_or_else(|err| {
                panic!(
                    "Unable to write to temporary file {:?} for editing: {}",
                    tmpfile, err
                )
            });

            let editor = editor.split_whitespace().collect::<Vec<_>>();

            let edit_exit_code = Command::new(editor[0])
                .args(&editor[1..])
                .arg(&tmpfile)
                .status()
                .unwrap_or_else(|err| panic!("Failed to execute {:?} editor: {}", editor, err));

            if edit_exit_code.success() {
                let new_content = std::fs::read_to_string(&tmpfile).unwrap_or_else(|err| {
                    panic!(
                        "Unable to read from temporary file {:?} after editing: {}",
                        tmpfile, err
                    )
                });
                items[index].contents.clone_from(&new_content);

                backend.save(&stack, items).unwrap();

                output.log(vec!["action", "item"], vec![vec!["Edited", &new_content]]);
            }
        }
    }
}

fn pick_indices(stack: String, indices: Vec<usize>, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        let mut seen: Vec<usize> = vec![];
        seen.reserve_exact(indices.len());
        let indices: Vec<usize> = indices.iter().map(|i| items.len() - 1 - i).rev().collect();
        for i in indices {
            if i > items.len() || seen.contains(&i) {
                // TODO: What should be the output here? Some stderr?
                // command.log("Pick", "ignoring out-of-bounds index");
                // command.log("Pick", "ignoring duplicate index");
                continue;
            }
            let i = i - seen.iter().filter(|j| j < &&i).count();
            let picked = items.remove(i);
            items.push(picked);
            seen.push(i);
        }

        backend.save(&stack, items).unwrap();

        if output.is_nonquiet_for_humans() {
            list_n_latest_items(stack, seen.len(), backend, output);
        }
    }
}

fn move_latest_item(source: String, dest: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&source) {
        let mut items = items;
        if let Some(item) = items.pop() {
            backend.save(&source, items).unwrap();

            output.log(
                vec!["action", "new-stack", "old-stack"],
                vec![vec!["Move", &dest, &source]],
            );

            push_item(dest, item, backend, &OutputFormat::Silent);
        }
    }
}

fn move_all_items(source: String, dest: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(src_items) = backend.load(&source) {
        let count = src_items.len();

        if !src_items.is_empty() {
            let all_items = match backend.load(&dest) {
                Ok(dest_items) => {
                    let mut all_items = dest_items;
                    for item in src_items {
                        all_items.push(item);
                    }
                    all_items
                }
                _ => src_items,
            };

            backend.save(&dest, all_items).unwrap();
            backend.save(&source, vec![]).unwrap();
        }

        output.log(
            vec!["action", "new-stack", "old-stack", "num-moved"],
            vec![vec!["Move All", &dest, &source, &count.to_string()]],
        );
    }
}

fn swap_latest_two_items(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;

        if items.len() < 2 {
            return;
        }

        let a = items.pop().unwrap();
        let b = items.pop().unwrap();
        items.push(a);
        items.push(b);

        backend.save(&stack, items).unwrap();

        if output.is_nonquiet_for_humans() {
            list_n_latest_items(stack, 2, backend, output);
        }
    }
}

fn rotate_latest_three_items(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;

        if items.len() < 3 {
            swap_latest_two_items(stack, backend, output);
            return;
        }

        let a = items.pop().unwrap();
        let b = items.pop().unwrap();
        let c = items.pop().unwrap();

        items.push(a);
        items.push(c);
        items.push(b);

        backend.save(&stack, items).unwrap();

        if output.is_nonquiet_for_humans() {
            list_n_latest_items(stack, 3, backend, output);
        }
    }
}

fn next_to_latest(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        if items.is_empty() {
            return;
        }
        let to_the_back = items.pop().unwrap();
        items.insert(0, to_the_back);

        backend.save(&stack, items).unwrap();

        if output.is_nonquiet_for_humans() {
            peek_latest_item(stack, backend, output);
        }
    }
}

fn peek_latest_item(stack: String, backend: &Backend, output: &OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    if let Ok(items) = backend.load(&stack) {
        let top_item = items.last().map(|i| i.contents.as_str());

        let output_it = |it| output.log_always(vec!["position", "item"], it);

        match top_item {
            Some(contents) => output_it(vec![vec!["Now", contents]]),
            None => {
                if output.is_nonquiet_for_humans() {
                    output_it(vec![vec!["Now", "NOTHING"]])
                } else {
                    output_it(vec![])
                }
            }
        }
    }
}

fn count_all_items(stack: String, backend: &Backend, output: &OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    if let Ok(items) = backend.load(&stack) {
        let len = items.len().to_string();
        output.log_always(vec!["items"], vec![vec![&len]])
    }
}

fn is_empty(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        if !items.is_empty() {
            output.log_always(vec!["empty"], vec![vec!["false"]]);
            // Exit with a failure (nonzero status) when not empty.
            // This helps people who do shell scripting do something like:
            //     while ! sigi -t $stack is-empty ; do <ETC> ; done
            // TODO: It would be better modeled as an error, if anyone uses as a lib this will surprise.
            if let OutputFormat::TerseText = output {
                return;
            } else {
                std::process::exit(1);
            }
        }
    }
    output.log_always(vec!["empty"], vec![vec!["true"]]);
}

fn list_stacks(backend: &Backend, output: &OutputFormat) {
    if let Ok(stacks) = backend.list_stacks() {
        let mut stacks = stacks;
        stacks.sort();
        let strs = stacks.iter().map(|stack| vec![stack.as_str()]).collect();
        output.log_always(vec!["stack"], strs);
    }
}

// ===== ListAll/Head/Tail =====

struct ListRange {
    stack: String,
    // Ignored if starting "from_end".
    start: usize,
    limit: Option<usize>,
    from_end: bool,
}

fn list_range(range: ListRange, backend: &Backend, output: &OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    if let Ok(items) = backend.load(&range.stack) {
        let limit = match range.limit {
            Some(n) => n,
            None => items.len(),
        };

        let start = if range.from_end {
            if limit <= items.len() {
                items.len() - limit
            } else {
                0
            }
        } else {
            range.start
        };

        let lines = items
            .into_iter()
            .rev()
            .enumerate()
            .skip(start)
            .take(limit)
            .map(|(i, item)| {
                // Pad human output numbers to line up nicely with "Now".
                let position = if output.is_nonquiet_for_humans() {
                    match i {
                        0 => "Now".to_string(),
                        1..=9 => format!("  {}", i),
                        10..=99 => format!(" {}", i),
                        _ => i.to_string(),
                    }
                } else {
                    i.to_string()
                };

                let created = item
                    .history
                    .iter()
                    .find(|(status, _)| status == "created")
                    .map(|(_, dt)| output.format_time(*dt))
                    .unwrap_or_else(|| "unknown".to_string());

                vec![position, item.contents, created]
            })
            .collect::<Vec<_>>();

        let labels = vec!["position", "item", "created"];

        if lines.is_empty() {
            if output.is_nonquiet_for_humans() {
                output.log(labels, vec![vec!["Now", "NOTHING"]]);
            }
            return;
        }

        // Get the lines into a "borrow" state (&str instead of String) to make log happy.
        let lines = lines
            .iter()
            .map(|line| line.iter().map(|s| s.as_str()).collect())
            .collect();

        output.log_always(labels, lines);
    }
}

fn list_all_items(stack: String, backend: &Backend, output: &OutputFormat) {
    let range = ListRange {
        stack,
        start: 0,
        limit: None,
        from_end: false,
    };

    list_range(range, backend, output);
}

fn list_n_latest_items(stack: String, n: usize, backend: &Backend, output: &OutputFormat) {
    let range = ListRange {
        stack,
        start: 0,
        limit: Some(n),
        from_end: false,
    };

    list_range(range, backend, output);
}

fn list_n_oldest_items(stack: String, n: usize, backend: &Backend, output: &OutputFormat) {
    let range = ListRange {
        stack,
        start: 0,
        limit: Some(n),
        from_end: true,
    };

    list_range(range, backend, output);
}

// ===== Helper functions =====

fn stack_history_of(stack: &str) -> String {
    stack.to_string() + HISTORY_SUFFIX
}
