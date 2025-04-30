// This is free and unencumbered software released into the public domain.

use near_api::{AccountId, NetworkConfig, Signer, SignerTrait as _};

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};

#[tokio::main]
pub async fn import(account_id: AccountId, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let network_name = match account_id.as_str().split(".").last() {
        Some("near") => "mainnet",
        Some("testnet") => "testnet",
        _ => {
            ceprintln!("<s,r>error:</> Unable to determine network name from the account");
            return Err(EX_DATAERR);
        }
    };

    let network_config = match network_name {
        "mainnet" => NetworkConfig::mainnet(),
        "testnet" => NetworkConfig::testnet(),
        _ => unreachable!(),
    };

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Checking for credentials in keychain...");
    }

    let keychain = Signer::from_keystore_with_search_for_keys(account_id.clone(), &network_config)
        .await
        .inspect_err(|error| {
            ceprintln!("<s,r>error:</> unable to find keys for the account: {error}")
        })
        .map_err(|_| EX_CONFIG)?;

    if let Err(error) = keychain.get_public_key() {
        ceprintln!("<s,r>error:</> couldn't access credentials in keychain: {error}");
        return Err(EX_SOFTWARE);
    };

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Found credentials in keychain");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Verifying account exists on network...");
    }

    if let Err(error) = near_api::Account(account_id.clone())
        .view()
        .fetch_from(&network_config)
        .await
    {
        ceprintln!("<s,r>error:</> account doesn't exist on the network: {error}");
        return Err(EX_SOFTWARE);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Verified account exists on network");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Saving account info locally...");
    }

    let Some(dir) = dirs::home_dir() else {
        ceprintln!("<s,r>error:</> Unable to determine home directory");
        return Err(EX_CONFIG);
    };

    let dir = dir
        .join(".asimov")
        .join("accounts")
        .join("near")
        .join(network_name);

    if let Err(error) = std::fs::create_dir_all(&dir) {
        ceprintln!("<s,r>error:</> failed to create directory for saving accounts: {error}");
        return Err(EX_CANTCREAT);
    }

    let account_file = dir.join(account_id.as_str());

    if account_file.exists() {
        if flags.verbose >= 1 {
            cprintln!(
                "<s,y>!</> Account already exists locally at {}",
                account_file.display()
            );
        }
        return Ok(());
    }

    if let Err(error) = std::fs::File::create(&account_file) {
        ceprintln!("<s,r>error:</> failed to save account: {error}");
        return Err(EX_CANTCREAT);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Imported account to {}", account_file.display());
    }

    Ok(())
}
