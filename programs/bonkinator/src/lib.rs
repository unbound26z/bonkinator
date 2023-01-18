use std::mem::size_of;

use address::BONK_MINT;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use error::BonkError;
pub mod address;
pub mod error;
declare_id!("AM11xAodtV57L3Zuo2y23WERsyASJtXTzpYh2oarqsh5");

#[account]
pub struct Tweet {
    pub tweet_id: String,
    pub owner: Option<Pubkey>,
    pub price: Option<u64>,
}

#[derive(Accounts)]
#[instruction(tweet_id: String)]
pub struct BuyTweet<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub buyer_bonk_acc: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds=[b"tweet", tweet_id.as_bytes()],
        bump,
        payer = buyer,
        space = 8 + size_of::<Tweet>()
    )]
    pub tweet: Account<'info, Tweet>,

    #[account(
        mut,
        seeds=[b"treasury", bonk_mint.key().as_ref()],
        bump,
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(address=BONK_MINT.parse::<Pubkey>().unwrap() @ BonkError::WrongBonkTokenMint)]
    #[account()]
    pub bonk_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateBonkTA<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds=[b"treasury", bonk_mint.key().as_ref()],
        bump,
        token::authority = treasury,
        token::mint = bonk_mint,
        
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(address=BONK_MINT.parse::<Pubkey>().unwrap() @ BonkError::WrongBonkTokenMint)]
    #[account()]
    pub bonk_mint: Box<Account<'info, Mint>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[program]
pub mod bonkinator {
    use super::*;

    pub fn buy_tweet<'a,'b,'c,'info>(ctx: Context<'a,'b,'c,'info,BuyTweet<'info>>, tweet_id: String) -> Result<()> {
        let tweet = &mut ctx.accounts.tweet;
        tweet.tweet_id = tweet_id;

        if let Some(owner) = tweet.owner {
            let remaining_accounts = &mut ctx.remaining_accounts.iter();
            let seller_bonk_acc = Account::<TokenAccount>::try_from(remaining_accounts.next().expect("Seller bonk token account not provided")).expect("Not a token account");
            
            require!(
                seller_bonk_acc.mint.key()==ctx.accounts.bonk_mint.key(),
                BonkError::NotABonkTokenAccount
            );

            require!(
                ctx.accounts.buyer.key()!=owner, 
                BonkError::AlreadyOwner
            );

            require!(
               seller_bonk_acc.owner.key() == owner,
                BonkError::WrongSellerTokenAccount
            );

            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.buyer_bonk_acc.to_account_info(),
                        to: seller_bonk_acc.to_account_info(),
                        authority: ctx.accounts.buyer.to_account_info(),
                    },
                ),
                tweet.price.unwrap() + (tweet.price.unwrap() / 10),
            )
            .map_err(|_| BonkError::NotEnoughBonk)?;

            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.buyer_bonk_acc.to_account_info(),
                        to: ctx.accounts.treasury.to_account_info(),
                        authority: ctx.accounts.buyer.to_account_info(),
                    },
                ),
                tweet.price.unwrap() / 10,
            )
            .map_err(|_| BonkError::NotEnoughBonk)?;

            tweet.owner = Some(ctx.accounts.buyer.key());
            tweet.price = Some(tweet.price.unwrap() + (tweet.price.unwrap() / 5))
        } else {
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.buyer_bonk_acc.to_account_info(),
                        to: ctx.accounts.treasury.to_account_info(),
                        authority: ctx.accounts.buyer.to_account_info(),
                    },
                ),
               100000000000,
            )
            .map_err(|_| BonkError::NotEnoughBonk)?;

            tweet.owner = Some(ctx.accounts.buyer.key());
            tweet.price = Some(100000000000);
        }

        Ok(())
    }

    pub fn create_bonk_token_account(_ctx: Context<CreateBonkTA>) -> Result<()> {

        
        Ok(())
    }
}
