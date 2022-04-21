use crate::data::{Backend, Item};
use crate::output::OutputFormat;

const HISTORY_SUFFIX: &str = "_history";

// TODO: Consider more shuffle words: https://docs.factorcode.org/content/article-shuffle-words.html

pub enum StackEffect {
    Push { stack: String, content: String },
    Complete { stack: String },
    Delete { stack: String },
    DeleteAll { stack: String },
    Pick { stack: String, indices: Vec<usize> },
    Move { stack: String, dest: String },
    MoveAll { stack: String, dest: String },
    Swap { stack: String },
    Rot { stack: String },
    Next { stack: String },
    Peek { stack: String },
    ListAll { stack: String },
    ListStacks,
    Head { stack: String, n: usize },
    Tail { stack: String, n: usize },
    Count { stack: String },
    IsEmpty { stack: String },
}

impl StackEffect {
    pub fn run(self, backend: &Backend, output: &OutputFormat) {
        match self {
            StackEffect::Push { stack, content } => push_content(stack, content, backend, output),
            StackEffect::Complete { stack } => complete_latest_item(stack, backend, output),
            StackEffect::Delete { stack } => delete_latest_item(stack, backend, output),
            StackEffect::DeleteAll { stack } => delete_all_items(stack, backend, output),
            StackEffect::Pick { stack, indices } => pick_indices(stack, indices, backend, output),
            StackEffect::Move { stack, dest } => move_latest_item(stack, dest, backend, output),
            StackEffect::MoveAll { stack, dest } => move_all_items(stack, dest, backend, output),
            StackEffect::Swap { stack } => swap_latest_two_items(stack, backend, output),
            StackEffect::Rot { stack } => rotate_latest_three_items(stack, backend, output),
            StackEffect::Next { stack } => next_to_latest(stack, backend, output),
            StackEffect::Peek { stack } => peek_latest_item(stack, backend, output),
            StackEffect::ListAll { stack } => list_all_items(stack, backend, output),
            StackEffect::ListStacks => list_stacks(backend, output),
            StackEffect::Head { stack, n } => list_n_latest_items(stack, n, backend, output),
            StackEffect::Tail { stack, n } => list_n_oldest_items(stack, n, backend, output),
            StackEffect::Count { stack } => count_all_items(stack, backend, output),
            StackEffect::IsEmpty { stack } => is_empty(stack, backend, output),
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

fn complete_latest_item(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        if let Some(item) = items.pop() {
            let mut item = item;
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

    output.when_for_humans(|| peek_latest_item(stack, backend, output));
}

fn delete_latest_item(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        let mut items = items;
        if let Some(item) = items.pop() {
            let mut item = item;
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

    output.when_for_humans(|| peek_latest_item(stack, backend, output));
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

        list_n_latest_items(stack, seen.len(), backend, output);
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

        // Now show the first two items in their new order.
        list_n_latest_items(stack, 2, backend, output);
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

        list_n_latest_items(stack, 3, backend, output);
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

        peek_latest_item(stack, backend, output);
    }
}

fn peek_latest_item(stack: String, backend: &Backend, output: &OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    if let Ok(items) = backend.load(&stack) {
        let top_item = items.last().map(|i| i.contents.as_str());

        let output_it = |it| output.log(vec!["position", "item"], it);

        match top_item {
            Some(contents) => output_it(vec![vec!["Now", contents]]),
            None => output.for_human_or_programmatic(
                || output_it(vec![vec!["Now", "NOTHING"]]),
                || output_it(vec![]),
            ),
        }
    }
}

fn count_all_items(stack: String, backend: &Backend, output: &OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    if let Ok(items) = backend.load(&stack) {
        let len = items.len().to_string();
        output.log(vec!["items"], vec![vec![&len]])
    }
}

fn is_empty(stack: String, backend: &Backend, output: &OutputFormat) {
    if let Ok(items) = backend.load(&stack) {
        if !items.is_empty() {
            output.log(vec!["empty"], vec![vec!["false"]]);
            // Exit with a failure (nonzero status) when not empty.
            // This helps people who do shell scripting do something like:
            //     while ! sigi -t $stack is-empty ; do <ETC> ; done
            // TODO: It would be better modeled as an error, if anyone uses as a lib this will surprise.
            std::process::exit(1);
        }
    }
    output.log(vec!["empty"], vec![vec!["true"]]);
}

fn list_stacks(backend: &Backend, output: &OutputFormat) {
    if let Ok(stacks) = backend.list_stacks() {
        let mut stacks = stacks;
        stacks.sort();
        let strs = stacks.iter().map(|stack| vec![stack.as_str()]).collect();
        output.log(vec!["stack"], strs);
    }
}

// ===== ListAll/Head/Tail =====

trait Listable {
    fn range(self) -> ListRange;
}

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
                let position = output.for_human_or_programmatic(
                    || match i {
                        0 => "Now".to_string(),
                        1..=9 => format!("  {}", i),
                        10..=99 => format!(" {}", i),
                        _ => i.to_string(),
                    },
                    || i.to_string(),
                );

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
            output.when_for_humans(|| output.log(labels, vec![vec!["Now", "NOTHING"]]));
            return;
        }

        // Get the lines into a "borrow" state (&str instead of String) to make log happy.
        let lines = lines
            .iter()
            .map(|line| line.iter().map(|s| s.as_str()).collect())
            .collect();

        output.log(labels, lines);
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
