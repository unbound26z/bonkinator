use address::{AUTHORITY_PUBKEY, BONK_MINT};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use error::BonkError;
pub mod address;
pub mod error;
declare_id!("AjWzDnEEKPYvANmYvSsmu7LDfATQjHkfjzK1LMDUQSzR");

const BONK_DECIMALS: u32 = 5;
const INITIAL_PRICE: u64 = 10_000;

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
    pub buyer_bonk_acc: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, seeds=[b"tweet", tweet_id.as_bytes()], bump, payer = buyer, space = 8 + 4 + tweet_id.len() + 33 + 9)]
    pub tweet: Box<Account<'info, Tweet>>,

    #[account(
        mut,
        seeds=[b"treasury", bonk_mint.key().as_ref()],
        bump,
    )]
    pub treasury: Box<Account<'info, TokenAccount>>,

    #[account(address=BONK_MINT.parse::<Pubkey>().unwrap() @ BonkError::WrongBonkTokenMint)]
    pub bonk_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateBonkTA<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account( init, payer = payer, seeds=[b"treasury", bonk_mint.key().as_ref()], bump, token::authority = treasury, token::mint = bonk_mint)]
    pub treasury: Box<Account<'info, TokenAccount>>,

    #[account(address=BONK_MINT.parse::<Pubkey>().unwrap() @ BonkError::WrongBonkTokenMint)]
    pub bonk_mint: Box<Account<'info, Mint>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnBonk<'info> {
    #[account(mut, address=AUTHORITY_PUBKEY.parse::<Pubkey>().unwrap() @ BonkError::NotAuthority)]
    pub payer: Signer<'info>,

    #[account(mut, seeds=[b"treasury", bonk_mint.key().as_ref()], bump, token::authority = treasury, token::mint = bonk_mint)]
    pub treasury: Box<Account<'info, TokenAccount>>,

    #[account(address=BONK_MINT.parse::<Pubkey>().unwrap() @ BonkError::WrongBonkTokenMint)]
    #[account(mut)]
    pub bonk_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[program]
pub mod bonkinator {
    use super::*;

    pub fn buy_tweet<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BuyTweet<'info>>,
        tweet_id: String,
    ) -> Result<()> {
        let tweet = &mut ctx.accounts.tweet;
        tweet.tweet_id = tweet_id;

        if let Some(owner) = tweet.owner {
            let remaining_accounts = &mut ctx.remaining_accounts.iter();
            let seller_bonk_acc = Account::<TokenAccount>::try_from(
                remaining_accounts
                    .next()
                    .expect("Seller bonk token account not provided"),
            )
            .expect("Not a token account");

            require!(
                seller_bonk_acc.mint.key() == ctx.accounts.bonk_mint.key(),
                BonkError::NotABonkTokenAccount
            );

            require!(ctx.accounts.buyer.key() != owner, BonkError::AlreadyOwner);

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
                tweet.price.unwrap() + (tweet.price.unwrap().checked_div(10).unwrap()),
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
                tweet.price.unwrap().checked_div(10).unwrap(),
            )
            .map_err(|_| BonkError::NotEnoughBonk)?;

            tweet.owner = Some(ctx.accounts.buyer.key());
            tweet.price = Some(
                tweet
                    .price
                    .unwrap()
                    .checked_add(tweet.price.unwrap().checked_div(5).unwrap())
                    .unwrap(),
            )
        } else {
            let price = INITIAL_PRICE
                .checked_mul(10_u64.checked_pow(BONK_DECIMALS).unwrap())
                .unwrap();
            anchor_spl::token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.buyer_bonk_acc.to_account_info(),
                        to: ctx.accounts.treasury.to_account_info(),
                        authority: ctx.accounts.buyer.to_account_info(),
                    },
                ),
                price,
            )
            .map_err(|_| BonkError::NotEnoughBonk)?;

            tweet.owner = Some(ctx.accounts.buyer.key());
            tweet.price = Some(price);
        }

        Ok(())
    }

    pub fn create_bonk_token_account(_ctx: Context<CreateBonkTA>) -> Result<()> {
        Ok(())
    }

    pub fn burn_bonk(ctx: Context<BurnBonk>, price: u64) -> Result<()> {
        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    from: ctx.accounts.treasury.to_account_info(),
                    authority: ctx.accounts.treasury.to_account_info(),
                    mint: ctx.accounts.bonk_mint.to_account_info(),
                },
                &[&[
                    b"treasury",
                    ctx.accounts.bonk_mint.key().as_ref(),
                    &[*ctx.bumps.get("treasury").unwrap()],
                ]],
            ),
            price,
        )
        .map_err(|_| BonkError::BurnError)?;

        Ok(())
    }
}
