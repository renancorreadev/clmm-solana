use anchor_lang::prelude::*;

/// Representa o estado do pool.
#[account]
pub struct Pool {
    /// Preço representado como a raiz quadrada em Q64.64.
    pub sqrt_price_x64: u128,
    /// Tick atual do pool.
    pub current_tick: i32,
    /// Liquidez global ativa na faixa atual.
    pub liquidity: u128,
}

impl Pool {
    // 16 bytes para u128 + 4 bytes para i32 + 16 bytes para u128.
    pub const LEN: usize = 16 + 4 + 16;
}

/// Representa um tick.
#[account]
pub struct Tick {
    /// Índice do tick.
    pub tick_index: i32,
    /// Delta líquido de liquidez ao cruzar este tick (pode ser negativo).
    pub liquidity_net: i128,
}

impl Tick {
    // 4 bytes para i32 + 16 bytes para i128.
    pub const LEN: usize = 4 + 16;
}

/// Representa a posição de liquidez do LP.
#[account]
pub struct Position {
    pub owner: Pubkey,
    /// Quantidade de liquidez depositada.
    pub liquidity_delta: u128,
    /// Tick inferior da faixa.
    pub lower_tick: i32,
    /// Tick superior da faixa.
    pub upper_tick: i32,
}

impl Position {
    // 32 bytes para Pubkey + 16 bytes para u128 + 4 + 4.
    pub const LEN: usize = 32 + 16 + 4 + 4;
}
