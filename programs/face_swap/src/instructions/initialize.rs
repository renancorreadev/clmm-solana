use crate::state::fee_collector::FeeCollector;
use crate::state::pool::Pool;
use crate::state::pool::Tick;
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

#[derive(Accounts)]
#[instruction(tick_index: i32)]
pub struct InitializeTick<'info> {
    #[account(init, payer = user, space = 8 + Tick::LEN)]
    pub tick: Account<'info, Tick>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
    let tick = &mut ctx.accounts.tick;
    tick.tick_index = tick_index;
    tick.liquidity_net = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeFeeCollector<'info> {
    #[account(init, payer = user, space = FeeCollector::LEN, seeds = [b"fee_collector"], bump)]
    pub fee_collector: Account<'info, FeeCollector>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_fee_collector(ctx: Context<InitializeFeeCollector>) -> Result<()> {
    let fee_collector = &mut ctx.accounts.fee_collector;
    fee_collector.fees = 0;
    Ok(())
}
