#![allow(unexpected_cfgs)]

use crate::errors::ErrorCode;
use crate::state::fee_collector::FeeCollector;
use crate::state::pool::Pool;
use crate::utils::math::compute_swap_step;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub fee_collector: Account<'info, FeeCollector>,
    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn swap(ctx: Context<Swap>, amount_in: u64, zero_for_one: bool) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    msg!(
        "Iniciando swap com amount_in = {} e zero_for_one = {}",
        amount_in,
        zero_for_one
    );
    let (new_sqrt_price, amount_in_consumed, amount_out, fee_amount) =
        compute_swap_step(pool.sqrt_price_x64, pool.liquidity, amount_in, zero_for_one)?;
    msg!("Swap step calculado: new_sqrt_price = {}, amount_in_consumed = {}, amount_out = {}, fee_amount = {}",
         new_sqrt_price, amount_in_consumed, amount_out, fee_amount);

    msg!(
        "Atualizando preço do pool: {} -> {}",
        pool.sqrt_price_x64,
        new_sqrt_price
    );
    pool.sqrt_price_x64 = new_sqrt_price;

    // Acumula a fee na conta fee_collector.
    let fee_collector = &mut ctx.accounts.fee_collector;
    fee_collector.fees = fee_collector
        .fees
        .checked_add(fee_amount)
        .ok_or(ErrorCode::Overflow)?;
    msg!(
        "Fee de {} acumulada. Total na fee_collector: {}",
        fee_amount,
        fee_collector.fees
    );

    msg!(
        "Swap concluído: in_consumed = {}, out = {}, fee = {}",
        amount_in_consumed,
        amount_out,
        fee_amount
    );
    Ok(())
}
