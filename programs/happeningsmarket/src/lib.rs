use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

declare_id!("7y3F8J4YDNAmXe4vk9Zgxs4HkYFzf4STziiLYjsoRUz");

#[program]
pub mod happeningsmarket {
    use super::*;

    // 1. Create a new Yes/No market
    pub fn create_market(ctx: Context<CreateMarket>,creationTime: u64,  question: String, ends_at: u64) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.creationTime = creationTime;
        market.question = question;
        market.ends_at = ends_at;
        market.yes_vault = ctx.accounts.yes_vault.key();
        market.no_vault = ctx.accounts.no_vault.key();
        market.state = MarketState::Open;
        market.total_yes = 0;
        market.total_no = 0;
        market.bump = ctx.bumps.market;

        Ok(())
    }

    // 2. User bets on YES or NO
    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, side: BetSide) -> Result<()> {
        let market = &mut ctx.accounts.market;

        require!(market.state == MarketState::Open, ErrorCode::MarketClosed);

        // Transfer tokens from user ATA → correct vault
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

        // Record user's bet (simple on-chain tracking)
        let user_bet = &mut ctx.accounts.user_bet;
        match side {
            BetSide::Yes => {
                user_bet.yes_amount = user_bet.yes_amount.checked_add(amount).unwrap();
                market.total_yes += amount;
            }
            BetSide::No => {
                user_bet.no_amount = user_bet.no_amount.checked_add(amount).unwrap();
                market.total_no += amount;
            }
        }
        user_bet.user = ctx.accounts.user.key();
        user_bet.market = market.key();

        Ok(())
    }
/*
    // 3. Resolve market (admin or oracle)
    pub fn resolve_market(ctx: Context<ResolveMarket>, winner: BetSide) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(market.state == MarketState::Open, ErrorCode::AlreadyResolved);
        market.state = MarketState::Resolved { winner };
        Ok(())
    }

    // 4. User claims winnings
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let market = &ctx.accounts.market;
        let user_bet = &ctx.accounts.user_bet;

        require!(matches!(market.state, MarketState::Resolved { winner } if winner != BetSide::None), ErrorCode::NotResolved);

        let (user_side_amount, total_winning_side, total_pool) = match market.state {
            MarketState::Resolved { winner: BetSide::Yes } => (user_bet.yes_amount, market.total_yes, market.total_yes + market.total_no),
            MarketState::Resolved { winner: BetSide::No } => (user_bet.no_amount, market.total_no, market.total_yes + market.total_no),
            _ => return err!(ErrorCode::NotResolved),
        };

        require!(user_side_amount > 0, ErrorCode::NothingToClaim);

        let payout = (user_side_amount as u128)
            .checked_mul(total_pool as u128)
            .unwrap()
            .checked_div(total_winning_side as u128)
            .unwrap() as u64;

        let remaining = payout.checked_sub(user_side_amount).unwrap();

        // Transfer from BOTH vaults to winner
        let seeds = &[b"market".as_ref(), &[market.bump]];
        let signer = &[&seeds[..]];

        // Transfer from Yes vault
        if market.total_yes > 0 {
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.yes_vault.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.market.to_account_info(),
                        mint: ctx.accounts.mint.to_account_info(),
                    },
                    signer,
                ),
                // Proportional amount
                (remaining as u128 * market.total_yes as u128 / total_pool as u128) as u64,
                ctx.accounts.mint.decimals,
            )?;
        }

        // Transfer from No vault (the rest)
        if market.total_no > 0 {
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.no_vault.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.market.to_account_info(),
                        mint: ctx.accounts.mint.to_account_info(),
                    },
                    signer,
                ),
                remaining.saturating_sub((remaining as u128 * market.total_yes as u128 / total_pool as u128) as u64),
                ctx.accounts.mint.decimals,
            )?;
        }

        // Mark claimed
        ctx.accounts.user_bet.yes_amount = 0;
        ctx.accounts.user_bet.no_amount = 0;

        Ok(())
    }
        */
}

// Accounts structs
#[derive(Accounts)]
#[instruction(creationTime: u64, question: String, ends_at: u64)] 
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + 208,
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
#[instruction(amount: u64, side: BetSide)]
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
    pub user_token_account: InterfaceAccount<'info, TokenAccount>, // User's ATA

    #[account(mut, constraint = vault_token_account.key() == if side == BetSide::Yes { market.yes_vault } else { market.no_vault })]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// ... other account structs (ResolveMarket, ClaimWinnings) similar

#[account]
pub struct Market {
    creationTime: u64, 
    pub question: String,
    pub ends_at: u64,
    pub yes_vault: Pubkey,
    pub no_vault: Pubkey,
    pub total_yes: u64,
    pub total_no: u64,
    pub state: MarketState,
    pub bump: u8,
}

#[account]
pub struct UserBet {
    pub user: Pubkey,
    pub market: Pubkey,
    pub yes_amount: u64,
    pub no_amount: u64,
}

impl UserBet {
    pub const LEN: usize = 32 + 32 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum BetSide { Yes, No }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MarketState {
    Open,
    Resolved { winner: BetSide },
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
}