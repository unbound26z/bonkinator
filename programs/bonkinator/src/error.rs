use anchor_lang::prelude::*;

#[error_code]
pub enum BonkError {
    #[msg("You don't have enough bonk!")]
    NotEnoughBonk,

    #[msg("Wrong seller token account!")]
    WrongSellerTokenAccount,

    #[msg("Wrong bonk token mint")]
    WrongBonkTokenMint,
}
