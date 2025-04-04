// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

mod feature;

use clientele::{
    crates::clap::{Parser, Subcommand as ClapSubcommand},
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

#[derive(Debug, ClapSubcommand)]
enum Command {}

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

    EX_OK
}
