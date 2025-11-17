use crate::parser::errors::JsonPathError;

pub const MAX: u32 = u32::MAX;

pub fn use_gas(gas: &mut u32, amount: u32) -> Result<(), JsonPathError> {
    if amount > *gas {
        return Err(JsonPathError::TookTooLong);
    }
    *gas -= amount;
    Ok(())
}
