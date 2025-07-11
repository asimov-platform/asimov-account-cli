// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

use asimov_account_cli::commands;

use clientele::{
    crates::clap::{Parser, Subcommand},
    StandardOptions,
    SysexitsError::{self, *},
};
use near_api::{AccountId, NearToken};

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
        /// The name of the account to find.
        #[clap(value_name = "NAME")]
        name: AccountId,
    },

    /// Import an existing ASIMOV account.
    Import {
        /// The name of the account to import.
        #[clap(value_name = "NAME")]
        name: AccountId,
    },

    /// List all known ASIMOV accounts.
    #[clap(alias = "ls")]
    List {},

    /// Register a new ASIMOV account.
    Register {
        /// The name of the account to register.
        #[clap(value_name = "NAME")]
        name: AccountId,

        /// The name of the account that sponsors the registration.
        #[clap(long, value_name = "NAME", requires = "sponsor_amount")]
        sponsor: Option<AccountId>,

        /// The amount of NEAR tokens to sponsor the account with. For example `10 NEAR`, `0.1 NEAR`, or `10 yoctoNEAR`.
        #[clap(long, value_name = "NEAR", requires = "sponsor")]
        sponsor_amount: Option<NearToken>,
    },

    /// Delete a registered ASIMOV account.
    #[clap(alias = "rm")]
    Delete {
        /// The name of the account to delete.
        #[clap(value_name = "NAME")]
        name: AccountId,

        /// The beneficiary account where remaining balance will be sent.
        #[clap(long, value_name = "NAME")]
        beneficiary: AccountId,
    },
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
        Command::Delete { name, beneficiary } => {
            commands::delete(name, beneficiary, &options.flags)
        }
        Command::Find { name } => commands::find(name, &options.flags),
        Command::Import { name } => commands::import(name, &options.flags),
        Command::List {} => commands::list(&options.flags),
        Command::Register {
            name,
            sponsor,
            sponsor_amount,
        } => commands::register(name, sponsor, sponsor_amount, &options.flags),
    };

    match result {
        Ok(()) => EX_OK,
        Err(err) => err,
    }
}
