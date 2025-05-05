// This is free and unencumbered software released into the public domain.

use crate::{
    network_name::NetworkName,
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::{ceprintln, cprintln};
use near_api::{Account, AccountId, Signer};

#[tokio::main]
pub async fn delete(
    account_id: AccountId,
    beneficiary: AccountId,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let network_name = NetworkName::try_from(&account_id).map_err(|_| {
        ceprintln!("<s,r>error:</> Unable to determine network name from the account");
        EX_DATAERR
    })?;
    let network_config = network_name.config();

    if flags.verbose >= 2 {
        cprintln!("<s,c>»</> Checking for credentials in keychain...");
    }

    let signer = Signer::from_keystore_with_search_for_keys(account_id.clone(), &network_config)
        .await
        .map_err(|error| {
            ceprintln!("<s,r>error:</> unable to find keys for the account: {error}");
            EX_SOFTWARE
        })
        .and_then(|signer| {
            Signer::new(signer).map_err(|error| {
                ceprintln!("<s,r>error:</> unable to find keys for the account: {error}");
                EX_SOFTWARE
            })
        })?;

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Found credentials in keychain");
    }

    if flags.verbose >= 2 {
        cprintln!("<s,g>»</> Sending delete request...");
    }

    let outcome = Account(account_id.clone())
        .delete_account_with_beneficiary(beneficiary)
        .with_signer(signer)
        .send_to(&network_config)
        .await
        .map_err(|error| {
            ceprintln!("<s,r>error:</> failed to delete account: {error}");
            EX_SOFTWARE
        })?;

    use near_api::near_primitives::views::FinalExecutionStatus;
    if let FinalExecutionStatus::Failure(error) = outcome.status {
        ceprintln!("<s,r>error:</> failed to delete account: {error}");
        return Err(EX_UNAVAILABLE);
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Delete request was successful");
    }

    let Some(dir) = dirs::home_dir() else {
        ceprintln!("Unable to determine home dir");
        return Err(EX_CONFIG);
    };
    let file = dir
        .join(".asimov")
        .join("accounts")
        .join("near")
        .join(network_name.as_str())
        .join(account_id.as_str());
    let moved_file = file.with_file_name(String::from(".") + account_id.as_str());
    match std::fs::rename(file, moved_file) {
        Ok(_) => (),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => {
            ceprintln!("<s,r>error:</> failed to remove account file: {error}");
            return Err(EX_SOFTWARE);
        }
    }

    if flags.verbose >= 1 {
        cprintln!("<s,g>✓</> Account <s>{account_id}</> has successfully been deleted");
    }

    Ok(())
}
