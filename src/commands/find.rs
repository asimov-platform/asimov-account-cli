// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use near_api::*;

#[tokio::main]
pub async fn find(account_name: &str, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let network = &NetworkConfig::testnet(); // TODO

    let keychain = Signer::from_keystore_with_search_for_keys(
        account_name.parse().expect("valid account name"),
        network,
    )
    .await
    .expect("keychain loaded");

    match keychain.get_public_key() {
        Ok(_) => {
            println!("yes");
            Ok(())
        }
        Err(error) => {
            if flags.debug {
                eprintln!("asimov: {}", error);
            }
            println!("no");
            Err(EX_NOUSER)
        }
    }
}
