use anchor_lang::prelude::*;

#[account]
pub struct FeeCollector {
    // CHECK: O campo `fees` é um `u64` que não precisa ser verificado
    pub fees: u64,
}

impl FeeCollector {
    pub const LEN: usize = 8 + 8;
}
