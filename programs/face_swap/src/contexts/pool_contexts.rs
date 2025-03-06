use crate::instructions::initialize::{initialize_pool, InitializePool};
use anchor_lang::prelude::*;

pub fn create_pool(
    ctx: Context<InitializePool>,
    sqrt_price_x64: u128,
    current_tick: i32,
) -> Result<()> {
    initialize_pool(ctx, sqrt_price_x64, current_tick)
}
