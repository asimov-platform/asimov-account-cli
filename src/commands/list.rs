// This is free and unencumbered software released into the public domain.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};
use near_api::AccountId;

pub fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    let Some(home_dir) = dirs::home_dir() else {
        ceprintln!("<s,r>error:</> Unable to determine home directory");
        return Err(EX_CONFIG);
    };

    let base_path = home_dir.join(".asimov").join("accounts").join("near");

    if !base_path.exists() {
        if flags.verbose >= 1 {
            cprintln!("No accounts found");
        }
        return Ok(());
    }

    if flags.verbose >= 2 {
        cprintln!(
            "<s,c>»</> Searching for accounts in {}",
            base_path.display()
        );
    }

    let mut networks: BTreeMap<String, BTreeSet<AccountId>> = BTreeMap::default();

    let dir = fs::read_dir(&base_path)
        .inspect_err(|error| {
            ceprintln!(
                "<s,r>error:</> failed to read accounts directory: {}",
                error
            )
        })
        .map_err(|_| EX_IOERR)?;

    for network in dir.flatten() {
        let network_path = network.path();
        if !network_path.is_dir() {
            continue;
        }

        let Some(network_name) = network_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let network_name = network_name.to_owned();

        let network_dir = fs::read_dir(&network_path)
            .inspect_err(|error| {
                ceprintln!("<s,r>error:</> failed to read network subdir: {error}")
            })
            .map_err(|_| EX_IOERR)?;

        for account in network_dir.flatten() {
            let account_id = account
                .file_name()
                .to_str()
                .ok_or(EX_CONFIG)?
                .parse()
                .map_err(|_| EX_CONFIG)?;
            networks
                .entry(network_name.clone())
                .or_default()
                .insert(account_id);
        }
    }

    if networks.is_empty() {
        if flags.verbose >= 1 {
            cprintln!("No accounts found");
        }
        return Ok(());
    }

    for (network_name, accounts) in networks {
        if accounts.is_empty() {
            continue;
        }
        cprintln!("<s,b>{network_name}</> accounts:");
        for account in accounts {
            cprintln!("  {account}")
        }
    }

    Ok(())
}
