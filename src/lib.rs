//! Sigi: An organizing CLI.
//!
//! The CLI and usage is documented briefly on the main GitHub project here:
//!
//! - https://github.com/sigi-cli/sigi
//!
//! Its "database" is currently little more than json files, and handles only
//! String values. It can work for research or small loads, but would be
//! sluggish for anything that needs to care about performance. Other data
//! stores like Redis and SQLite are planned.
//!
//! Other internals are documented, but the project is early in development
//! and should be considered **unstable** at best.

// TODO: Add guidance and examples for using sigi as a library... Or stop being a library.

/// The main interface of Sigi, stack (and stack-adjacent) actions.
pub mod effects;

/// The CLI implementation.
pub mod cli;

/// The item, stack, and persistence implementation.
pub mod data;

/// The printing implementation.
pub mod output;
