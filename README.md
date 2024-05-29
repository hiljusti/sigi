[<img align='right' src='https://sigi-cli.org/img/sigi.png' height='256' width='256'>](https://sigi-cli.org)

[![crates.io version](https://img.shields.io/crates/v/sigi)](https://crates.io/crates/sigi)
[![crates.io downloads](https://img.shields.io/crates/d/sigi?label=crates.io%20downloads)](https://crates.io/crates/sigi)
[![docs.rs docs](https://docs.rs/mio/badge.svg)](https://docs.rs/sigi)
[![discord badge](https://img.shields.io/discord/1141777454164365382?logo=discord)](https://discord.gg/Yehv682GJ4)

# Sigi CLI

`sigi` is an organizing tool for terminal lovers who hate organizing

Use `sigi` as extra memory. Use it to toss your tasks, groceries, or the next
board games you want to play onto a stack. Shell aliases are encouraged to
organize your various stacks.

---

```console
$ sigi -h
An organizing tool for terminal lovers who hate organizing

Usage: sigi [OPTIONS] [COMMAND]

Commands:
  interactive  Run in an interactive mode [aliases: i]
  -            Read input lines from standard input. Same commands as interactive mode, but only prints for printing commands. Intended for use in unix pipes
  complete     Move the current item to "<STACK>_history" and mark as completed [aliases: done, finish, fulfill]
  count        Print the total number of items in the stack [aliases: size, length]
  delete       Move the current item to "<STACK>_history" and mark as deleted [aliases: pop, remove, cancel, drop]
  delete-all   Move all items to "<STACK>_history" and mark as deleted [aliases: purge, pop-all, remove-all, cancel-all, drop-all]
  edit         Edit the content of an item. Other metadata like creation date is left unchanged
  head         Print the first N items (default is 10) [aliases: top, first]
  is-empty     Print "true" if stack has zero items, or print "false" (and exit with a nonzero exit code) if the stack does have items [aliases: empty]
  list         Print all items [aliases: ls, snoop, all]
  list-stacks  Print all stacks [aliases: stacks]
  move         Move current item to another stack
  move-all     Move all items to another stack
  next         Cycle to the next item; the current item becomes last [aliases: later, cycle, bury]
  peek         Print the first item. This is the default CLI behavior when no command is given [aliases: show]
  pick         Move items to the top of stack by their number
  push         Create a new item [aliases: create, add, do, start, new]
  rot          Rotate the three most-current items [aliases: rotate]
  swap         Swap the two most-current items
  tail         Print the last N items (default is 10) [aliases: bottom, last]
  help         Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet                    Omit any leading labels or symbols. Recommended for use in shell scripts
  -s, --silent                   Omit any output at all
  -v, --verbose                  Print more information, like when an item was created [aliases: noisy]
  -f, --format <FORMAT>          Use a programmatic format. Options include [csv, json, json-compact, tsv]. Not compatible with quiet/silent/verbose [possible values: csv, json, json-compact, tsv]
  -t, --stack <STACK>            Manage items in a specific stack [aliases: topic, about, namespace]
  -d, --data-store <DATA_STORE>  (Advanced) Manage sigi stacks in a specific directory. The default is either the value of a SIGI_HOME environment variable or your OS-specific home directory [aliases: dir, directory, store]
  -h, --help                     Print help (see more with '--help')
  -V, --version                  Print version

INTERACTIVE MODE:

Use subcommands in interactive mode directly. No OPTIONS (flags) are understood in interactive mode. The ; character can be used to separate commands.

The following additional commands are available:
    ?               Show the short version of "help"
    clear           Clear the terminal screen
    use             Change to the specified stack [aliases: stack]
    exit            Quit interactive mode [aliases: quit, q]
```

# Examples

## `sigi` as a to-do list

`sigi` can understand `do` (create a task) and `done` (complete a task).

```
$ alias todo='sigi --stack todo'

$ todo do Write some code
Creating: Write some code

$ todo do Get a drink
Creating: Get a drink

$ todo do Take a nap
Creating: Take a nap

$ todo list
Now: Take a nap
  1: Get a drink
  2: Write some code

$ sleep 20m

$ todo done
Completed: Take a nap
```

It's best to use `sigi` behind a few aliases with unique "stacks". You should
save these aliases in your `~/.bashrc` or `~/.zshrc` or whatever your shell has
for configuration. `sigi` accepts a `--stack` option, and you can have as many
stacks as you can think of names.

Forgot what to do next?

```
$ todo
Now: Get a drink
```

Not going to do it?

```
$ todo delete
Deleted: Get a drink
```

## `sigi` as a save-anything list

Extending the alias idea, you can use `sigi` to store anything you want to
remember later.

```
$ alias watch-later='sigi --stack watch-later'

$ watch-later add One Punch Man
Creating: One Punch Man
```

```
$ alias story-ideas='sigi --stack=story-ideas'

$ story-ideas add Alien race lives backwards through time.
Creating: Alien race lives backwards through time.
```

## `sigi` remote via ssh

If you have a host you can access remotely, using a tool like
[OpenSSH](https://www.openssh.com), you can also use sigi across machines.
Consider using an alias like this:

```
$ alias home-todo='ssh -qt user@host.or.ip sigi --stack=home-todo'
```

> Protip: If you do a bunch of machine hopping via SSH, consider adding host
aliases in [`$HOME/.ssh/config`](https://man.openbsd.org/ssh_config.5). I set
these up something like this:
> ```
> Host hq
>     User boonieppper
>     HostName 192.168.x.x
>     IdentityFile ~/.ssh/etc
> ```
> which allows for just running `ssh hq`, for example.

## `sigi` as a local stack-based database

`sigi` understands the programmer-familiar `push` and `pop` idioms. It can be
used for simple, persistent, small-scale stack use-cases.

Using the `--quiet` (or `-q`) flag is recommended for shell scripts, as it
leaves out any leading labels or symbols. If used with a pipe, it's recommended
to use the `-` subcommand to read from standard input and only print if the
action requested is a printing action (like `list`).

`sigi` is pretty fast: sub-millisecond for basic use cases. That said, it is
not intended to handle large amounts of data, or concurrent throughput. For
something beefier with stack semantics, check out Redis.

# Installing

[![Packaging status](https://repology.org/badge/vertical-allrepos/sigi.svg)](https://repology.org/project/sigi/versions)

If your packaging system doesn't have it yet, the best way to install `sigi` is
through the Rust language package manager, `cargo`:

```
cargo install sigi
```

Instructions on installing `cargo` can be found here:

- https://doc.rust-lang.org/cargo/getting-started/installation.html

Please package it up for your Linux/BSD/etc distribution.

# Contributing and support

Please [open an issue](https://github.com/sigi-cli/sigi/issues) if you see
bugs or have ideas!

I'm looking for people to use [the `sigi` wiki](https://github.com/sigi-cli/sigi/wiki)
to share their tips, tricks, and examples.

Thanks for checking it out!
