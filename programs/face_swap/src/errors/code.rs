use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Faixa de ticks inválida")]
    InvalidTickRange,
    #[msg("Overflow aritmético")]
    Overflow,
    #[msg("Liquidez zero")]
    ZeroLiquidity,
}
