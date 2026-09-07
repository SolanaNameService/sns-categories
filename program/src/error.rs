use {num_derive::FromPrimitive, solana_program::program_error::ProgramError, thiserror::Error};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum SnsCategoriesError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("Numerical overflow")]
    Overflow,
}

impl From<SnsCategoriesError> for ProgramError {
    fn from(e: SnsCategoriesError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
