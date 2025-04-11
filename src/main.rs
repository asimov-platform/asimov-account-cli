// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

mod commands;

use clientele::{
    crates::clap::{Parser, Subcommand},
    StandardOptions,
    SysexitsError::{self, *},
};

/// ASIMOV Account Command-Line Interface (CLI)
#[derive(Debug, Parser)]
#[command(name = "asimov-account", long_about)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check whether an account exists on the network.
    Find {
        /// The name of the account to import.
        #[clap(value_name = "NAME")]
        name: String,
    },

    /// TBD
    Import {},

    /// TBD
    #[clap(alias = "ls")]
    List {},

    /// TBD
    #[cfg(feature = "unstable")]
    Register {},
}

pub fn main() -> SysexitsError {
    // Load environment variables from `.env`:
    clientele::dotenv().ok();

    // Expand wildcards and @argfiles:
    let Ok(args) = clientele::args_os() else {
        return EX_USAGE;
    };

    // Parse command-line options:
    let options = Options::parse_from(&args);

    // Print the version, if requested:
    if options.flags.version {
        println!("asimov-account {}", env!("CARGO_PKG_VERSION"));
        return EX_OK;
    }

    // Print the license, if requested:
    if options.flags.license {
        print!("{}", include_str!("../UNLICENSE"));
        return EX_OK;
    }

    // Configure debug output:
    if options.flags.debug {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Execute the given command:
    let result = match options.command.unwrap() {
        Command::Find { name } => commands::find(&name, &options.flags),
        Command::Import {} => commands::import(&options.flags),
        Command::List {} => commands::list(&options.flags),
        #[cfg(feature = "unstable")]
        Command::Register {} => commands::register(&options.flags),
    };

    match result {
        Ok(()) => EX_OK,
        Err(err) => err,
    }
}
