// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};
use near_api::{AccountId, NetworkConfig, Signer, SignerTrait as _};

#[tokio::main]
pub async fn find(account_id: AccountId, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let network_name = match account_id.as_str().split(".").last() {
        Some("near") => "mainnet",
        Some("testnet") => "testnet",
        _ => {
            ceprintln!("<s,r>error:</> unable to determine network name from the account");
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
        .map_err(|error| {
            ceprintln!("<s,r>error:</> unable to find keys for the account: {error}");
            EX_CONFIG
        })?;

    if let Err(error) = keychain.get_public_key() {
        ceprintln!("<s,r>error:</> couldn't access credentials in keychain: {error}");
        return Err(EX_SOFTWARE);
    };

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Found credentials in keychain");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,g>»</> Checking account exists on the network");
    }

    if let Err(error) = near_api::Account(account_id.clone())
        .view()
        .fetch_from(&network_config)
        .await
    {
        ceprintln!("<s,r>error:</> account was found locally but doesn't seem to exist on the network: {error}");
        return Err(EX_SOFTWARE);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Confirmed account exists");
    }

    cprintln!("<s,g>✓</> Account <s>{account_id}</> is valid and exists on the network");

    Ok(())
}
