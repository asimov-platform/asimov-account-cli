// This is free and unencumbered software released into the public domain.

use near_api::AccountId;

#[derive(Debug, Clone, Copy)]
pub enum NetworkName {
    Testnet,
    Mainnet,
}

impl NetworkName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Testnet => "testnet",
            Self::Mainnet => "mainnet",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NetworkNameError {
    UnknownNetwork,
}

impl TryFrom<&AccountId> for NetworkName {
    type Error = NetworkNameError;

    fn try_from(value: &AccountId) -> Result<Self, Self::Error> {
        match value.as_str().split(".").last() {
            Some("near") => Ok(Self::Mainnet),
            Some("testnet") => Ok(Self::Testnet),
            _ => Err(NetworkNameError::UnknownNetwork),
        }
    }
}

impl std::fmt::Display for NetworkName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
