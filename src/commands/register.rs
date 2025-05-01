// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};
use near_api::{AccountId, NetworkConfig};
use near_crypto::PublicKey;

#[tokio::main]
pub async fn register(account_id: AccountId, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let key_pair_properties = near_cli_rs::common::generate_keypair()
        .inspect_err(|error| ceprintln!("<s,r>error:</> failed to generate credentials: {error}"))
        .map_err(|_| EX_SOFTWARE)?;
    let public_key: PublicKey = key_pair_properties
        .public_key_str
        .parse()
        .inspect_err(|error| ceprintln!("<s,r>error:</> failed to generate credentials: {error}"))
        .map_err(|_| EX_SOFTWARE)?;
    let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
    let config = near_cli_rs::config::Config::default();
    let network_name = match account_id.as_str().split(".").last() {
        Some("near") => "mainnet",
        Some("testnet") => "testnet",
        _ => {
            ceprintln!(
                "<s,r>error:</> unable to determine network name from the account <s>{account_id}</>"
            );
            return Err(EX_USAGE);
        }
    };
    let api_network_config = match network_name {
        "mainnet" => NetworkConfig::mainnet(),
        "testnet" => NetworkConfig::testnet(),
        _ => unreachable!(),
    };
    let Some(cli_network_config) = config.network_connection.get(network_name) else {
        return Err(EX_SOFTWARE);
    };

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Sending registration request...");
    }

    let result = near_api::Account::create_account(account_id.clone())
        .sponsor_by_faucet_service()
        .public_key(public_key)
        .map_err(|_| SysexitsError::EX_UNAVAILABLE)?
        .send_to_config_faucet(&api_network_config)
        .await
        .map_err(|_| SysexitsError::EX_TEMPFAIL)?;

    use near_api::near_primitives::views::{FinalExecutionOutcomeView, FinalExecutionStatus};
    let outcome: FinalExecutionOutcomeView = result
        .json()
        .await
        .inspect_err(|error| {
            ceprintln!("<s,r>error:</> failed to parse response: {error}");
        })
        .map_err(|_| EX_SOFTWARE)?;

    // Check for explicit failure. The returned status could also be `NotStarted` so we confirm
    // that it was created below.
    if let FinalExecutionStatus::Failure(error) = outcome.status {
        ceprintln!("<s,r>error:</> failed to create account: {error}");
        return Err(EX_UNAVAILABLE);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Sent registration request");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Confirming account exists...");
    }

    if let Err(error) = near_api::Account(account_id.clone())
        .view()
        .fetch_from(&api_network_config)
        .await
    {
        ceprintln!("<s,r>error:</> account does not seem to exist: {error}");
        return Err(EX_SOFTWARE);
    }
    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Confirmed account exists");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Saving credentials to keychain...");
    }

    if let Err(error) = near_cli_rs::common::save_access_key_to_keychain(
        cli_network_config.clone(),
        &key_pair_properties_buf,
        &key_pair_properties.public_key_str,
        account_id.as_str(),
    ) {
        ceprintln!("<s,r>error:</> failed to save credentials to keychain: {error}");
        return Err(EX_SOFTWARE);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Saved credentials to keychain");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Saving account info locally...");
    }

    let Some(dir) = dirs::home_dir() else {
        ceprintln!("Unable to determine home dir");
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

    if let Err(error) = std::fs::File::create(&account_file) {
        ceprintln!("<s,r>error:</> failed to save account: {error}");
        return Err(EX_CANTCREAT);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Saved account to {}", account_file.display());
    }

    Ok(())
}
