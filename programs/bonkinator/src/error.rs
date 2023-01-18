use anchor_lang::prelude::*;

#[error_code]
pub enum BonkError {
    #[msg("You don't have enough bonk!")]
    NotEnoughBonk,

    #[msg("Wrong seller token account!")]
    WrongSellerTokenAccount,

    #[msg("Wrong bonk token mint")]
    WrongBonkTokenMint,

    #[msg("You already own the tweet")]
    AlreadyOwner,

    #[msg("Remaining account isn't sellers token account")]
    NotABonkTokenAccount,
}
