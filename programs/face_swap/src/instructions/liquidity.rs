use crate::errors::code::ErrorCode;
use crate::state::pool::{Pool, Position, Tick};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(init, payer = user, space = 8 + Position::LEN)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub lower_tick_account: Account<'info, Tick>,
    #[account(mut)]
    pub upper_tick_account: Account<'info, Tick>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    liquidity_delta: u128,
    lower_tick: i32,
    upper_tick: i32,
) -> Result<()> {
    if lower_tick >= upper_tick {
        return Err(ErrorCode::InvalidTickRange.into());
    }

    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.user.key();
    position.liquidity_delta = liquidity_delta;
    position.lower_tick = lower_tick;
    position.upper_tick = upper_tick;

    let pool = &mut ctx.accounts.pool;
    if pool.current_tick >= lower_tick && pool.current_tick < upper_tick {
        pool.liquidity = pool
            .liquidity
            .checked_add(liquidity_delta)
            .ok_or(ErrorCode::Overflow)?;
    }

    let lower_tick_account = &mut ctx.accounts.lower_tick_account;
    lower_tick_account.tick_index = lower_tick;
    lower_tick_account.liquidity_net = lower_tick_account
        .liquidity_net
        .checked_add(liquidity_delta as i128)
        .ok_or(ErrorCode::Overflow)?;
    let upper_tick_account = &mut ctx.accounts.upper_tick_account;
    upper_tick_account.tick_index = upper_tick;
    upper_tick_account.liquidity_net = upper_tick_account
        .liquidity_net
        .checked_sub(liquidity_delta as i128)
        .ok_or(ErrorCode::Overflow)?;
    Ok(())
}
