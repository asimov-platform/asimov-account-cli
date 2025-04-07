// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};

pub fn import(_flags: &StandardOptions) -> Result<(), SysexitsError> {
    Err(EX_UNAVAILABLE) // TODO
}
