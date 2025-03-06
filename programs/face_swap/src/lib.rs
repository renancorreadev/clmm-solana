#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod contexts;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("Evz3EUt4ZgE7AmuiX5VsgZ2fvdtGnqey2QSb6o9navSz");

#[program]
pub mod face_swap {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        sqrt_price_x64: u128,
        current_tick: i32,
    ) -> Result<()> {
        instructions::initialize::initialize_pool(ctx, sqrt_price_x64, current_tick)
    }

    pub fn initialize_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
        instructions::initialize::initialize_tick(ctx, tick_index)
    }

    pub fn initialize_fee_collector(ctx: Context<InitializeFeeCollector>) -> Result<()> {
        instructions::initialize::initialize_fee_collector(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        liquidity_delta: u128,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<()> {
        instructions::liquidity::add_liquidity(ctx, liquidity_delta, lower_tick, upper_tick)
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, zero_for_one: bool) -> Result<()> {
        instructions::swap::swap(ctx, amount_in, zero_for_one)
    }
}
