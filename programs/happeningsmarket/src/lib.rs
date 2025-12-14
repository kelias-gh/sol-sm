use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

declare_id!("EUYYLWEqZrwHE7votMDDPMeCW4NtvYYeEGQ3SgF1Ljzp");

/*
This program needs to be able to:
Create a market with Market struct that keeps track of amounts & state
Yes Vault
No Vault
Buy amount
Vault account user ATA
Sell
Resolve the market
*/

#[program]
pub mod happeningsmarket {
    use super::*;

    pub fn create_market(ctx: Context<CreateMarket>, creationTime: u64, endsAt: u64) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.creationTime = creationTime;
        market.ends_at = endsAt;
        market.yes_vault = ctx.accounts.yes_vault.key();
        market.no_vault = ctx.accounts.no_vault.key();
        market.state = MarketState::OPEN;
        market.total_yes = 0;
        market.total_no = 0;
        market.bump = ctx.bumps.market;
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, side: u8) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(market.state == MarketState::OPEN, ErrorCode::MarketClosed);

        require!(side == BetSide::YES || side == BetSide::NO, ErrorCode::InvalidBetSide);

        let transfer_cpi = TransferChecked {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };

        transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_cpi),
            amount,
            ctx.accounts.mint.decimals,
        )?;

        let user_bet = &mut ctx.accounts.user_bet;

        user_bet.bump = ctx.bumps.user_bet;

        if side == BetSide::YES {
            user_bet.yes_amount = user_bet.yes_amount.checked_add(amount).unwrap();
            market.total_yes += amount;
        } else {
            user_bet.no_amount = user_bet.no_amount.checked_add(amount).unwrap();
            market.total_no += amount;
        }

        user_bet.user = ctx.accounts.user.key();
        user_bet.market = market.key();

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(creationTime: u64, endsAt: u64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + Market::LEN,
        seeds = [b"market", creator.key().as_ref(), creationTime.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = creator,
        token::mint = mint,
        token::authority = market,
        seeds = [b"yes", market.key().as_ref()],
        bump
    )]
    pub yes_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        token::mint = mint,
        token::authority = market,
        seeds = [b"no", market.key().as_ref()],
        bump
    )]
    pub no_vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64, side: u8)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        init_if_needed,
        space = 8 + UserBet::LEN,
        payer = user,
        seeds = [b"bet", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_bet: Account<'info, UserBet>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// ... other account structs (ResolveMarket, ClaimWinnings) similar

#[account]
pub struct Market {
    pub creationTime: u64,
    pub ends_at: u64,
    pub yes_vault: Pubkey,
    pub no_vault: Pubkey,
    pub total_yes: u64,
    pub total_no: u64,
    pub state: u8,
    pub bump: u8,
}

#[account]
pub struct UserBet {
    pub user: Pubkey,
    pub market: Pubkey,
    pub yes_amount: u64,
    pub no_amount: u64,
    pub bump: u8
}

impl Market {
    pub const LEN: usize = 8 + 8 + 32 + 32 + 8 + 8 + 1 + 1;
}

impl UserBet {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1;
}

// Constants instead of enums - use these for comparisons
pub struct BetSide;
impl BetSide {
    pub const YES: u8 = 0;
    pub const NO: u8 = 1;
}

pub struct MarketState;
impl MarketState {
    pub const OPEN: u8 = 0;
    pub const RESOLVED: u8 = 1;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Market is closed")]
    MarketClosed,
    #[msg("Market has ended")]
    MarketEnded,
    #[msg("Market already resolved")]
    AlreadyResolved,
    #[msg("Market not resolved")]
    NotResolved,
    #[msg("Nothing to claim")]
    NothingToClaim,
    #[msg("Invalid bet side (must be 0 for Yes or 1 for No)")]
    InvalidBetSide,
}