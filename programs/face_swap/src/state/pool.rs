use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: u128,
}

impl Pool {
    pub const LEN: usize = 16 + 4 + 16;
}

#[account]
pub struct Tick {
    pub tick_index: i32,
    pub liquidity_net: i128,
}

impl Tick {
    pub const LEN: usize = 4 + 16;
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub liquidity_delta: u128,
    pub lower_tick: i32,
    pub upper_tick: i32,
}

impl Position {
    pub const LEN: usize = 32 + 16 + 4 + 4;
}

#[account]
pub struct FeeCollector {
    pub fees: u64,
}

impl FeeCollector {
    pub const LEN: usize = 8;
}
