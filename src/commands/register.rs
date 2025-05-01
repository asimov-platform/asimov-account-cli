// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};
use near_api::{AccountId, NearToken, NetworkConfig, Signer};
use near_crypto::PublicKey;

#[tokio::main]
pub async fn register(
    account_id: AccountId,
    sponsor: Option<AccountId>,
    sponsor_amount: Option<NearToken>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let key_pair_properties = near_cli_rs::common::generate_keypair().map_err(|error| {
        ceprintln!("<s,r>error:</> failed to generate credentials: {error}");
        EX_SOFTWARE
    })?;
    let public_key: PublicKey = key_pair_properties
        .public_key_str
        .parse()
        .map_err(|error| {
            ceprintln!("<s,r>error:</> failed to generate credentials: {error}");
            EX_SOFTWARE
        })?;
    let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
    let config = near_cli_rs::config::Config::default();
    let (network_name, api_network_config) = match account_id.as_str().split(".").last() {
        Some("near") => (NetworkName::Mainnet, NetworkConfig::mainnet()),
        Some("testnet") => (NetworkName::Testnet, NetworkConfig::testnet()),
        _ => {
            ceprintln!(
                "<s,r>error:</> unable to determine network name from the account <s>{account_id}</>. The account must end with either <s>.near</> for mainnet or <s>.testnet</> for testnet accounts.",
            );
            return Err(EX_USAGE);
        }
    };
    let Some(cli_network_config) = config.network_connection.get(network_name.as_str()) else {
        return Err(EX_SOFTWARE);
    };

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Sending registration request...");
    }

    use near_api::near_primitives::views::{FinalExecutionOutcomeView, FinalExecutionStatus};
    let outcome: FinalExecutionOutcomeView = match (&network_name, sponsor, sponsor_amount) {
        (NetworkName::Testnet, None, None) => {
            let result = near_api::Account::create_account(account_id.clone())
                .sponsor_by_faucet_service()
                .public_key(public_key)
                .map_err(|_| SysexitsError::EX_SOFTWARE)?
                .send_to_config_faucet(&api_network_config)
                .await
                .map_err(|error| {
                    ceprintln!("<s,r>error:</> failed to create account: {error}");
                    SysexitsError::EX_TEMPFAIL
                })?;

            result.json().await.map_err(|error| {
                ceprintln!("<s,r>error:</> failed to parse response: {error}");
                EX_SOFTWARE
            })?
        }
        (_, Some(sponsor), Some(amount)) => {
            let signer =
                Signer::from_keystore_with_search_for_keys(sponsor.clone(), &api_network_config)
                    .await
                    .map_err(|error| {
                        ceprintln!(
                            "<s,r>error:</> unable to find keys for the sponsor account: {error}"
                        );
                        EX_SOFTWARE
                    })
                    .and_then(|signer| {
                        Signer::new(signer).map_err(|error| {
                            ceprintln!(
                        "<s,r>error:</> unable to find keys for the sponsor account: {error}"
                    );
                            EX_SOFTWARE
                        })
                    })?;

            near_api::Account::create_account(account_id.clone())
                .fund_myself(sponsor, amount)
                .public_key(public_key)
                .map_err(|error| {
                    ceprintln!(
                        "<s,r>error:</> unexpected error while creating transaction: {error}"
                    );
                    SysexitsError::EX_SOFTWARE
                })?
                .with_signer(signer)
                .send_to(&api_network_config)
                .await
                .map_err(|error| {
                    ceprintln!("<s,r>error:</> failed to create account: {error}");
                    SysexitsError::EX_TEMPFAIL
                })?
        }
        (_, Some(_), None) | (_, None, Some(_)) => {
            ceprintln!("<s,r>error:</> options <s>--sponsor</> and <s>--sponsor-amount</> are required together");
            return Err(EX_USAGE);
        }
        (NetworkName::Mainnet, _, _) => {
            ceprintln!(
                "<s,r>error:</> mainnet account registration requires a sponsor and an amount to be specified (<s>--sponsor</> and <s>--sponsor-amount</>)"
            );
            return Err(EX_USAGE);
        }
    };

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
        .join(network_name.as_str());
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

#[derive(Debug, Clone, Copy)]
enum NetworkName {
    Testnet,
    Mainnet,
}

impl NetworkName {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Testnet => "testnet",
            Self::Mainnet => "mainnet",
        }
    }
}

impl std::fmt::Display for NetworkName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
