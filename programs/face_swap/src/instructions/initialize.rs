use crate::state::pool::Pool;
use crate::state::tick::Tick;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + Pool::LEN)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    sqrt_price_x64: u128,
    current_tick: i32,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.sqrt_price_x64 = sqrt_price_x64;
    pool.current_tick = current_tick;
    pool.liquidity = 0;
    Ok(())
}
