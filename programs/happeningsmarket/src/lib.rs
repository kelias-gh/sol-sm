use anchor_lang::prelude::*;

declare_id!("7y3F8J4YDNAmXe4vk9Zgxs4HkYFzf4STziiLYjsoRUz"); // Change this with your actual program ID after deploy

#[program]

pub mod happeningsmarket {

    use super::*;

    pub fn create_market(ctx: Context<CreateMarket>, id: u32, num_bets: u8) -> Result<()> {
        let market = &mut ctx.accounts.market;

        market.id = id;

        market.authority = ctx.accounts.signer.key();

        market.bump = ctx.bumps.market;

        market.total_bets = num_bets;

        Ok(())
    }

    pub fn place_bet(
        ctx: Context<PlaceBet>,

        bet_idx: u8,

        yes: bool,

        no: bool,

        amount: u32,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;

        if yes {
            market.yes_total[bet_idx as usize] += amount;
        }

        if no {
            market.no_total[bet_idx as usize] += amount;
        }

        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(id: u32, num_bets: u8)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = signer,
        space = Market::LEN,
        seeds = [b"market", signer.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bet_idx: u8, yes: bool, no: bool, amount: u32)]

pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    pub signer: Signer<'info>,
}

#[account]

pub struct Market {
    pub id: u32,

    pub authority: Pubkey,

    pub total_bets: u8,

    pub bump: u8,

    pub yes_total: [u32; 10],

    pub no_total: [u32; 10],

    pub outcome: u8,
}

impl Market {
    pub const LEN: usize = 8 + 4 + 64 + 1 + 1 + ((4 * 10) * 2) + 1;
}
