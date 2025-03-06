use crate::errors::code::ErrorCode;
use anchor_lang::prelude::*;
use primitive_types::U256;

/// Calcula o swap step com maior precisão usando U256.
pub fn compute_swap_step(
    sqrt_price_current: u128,
    liquidity: u128,
    amount_in: u64,
    zero_for_one: bool,
) -> Result<(u128, u64, u64, u64)> {
    let fee_numer: u64 = 9970;
    let fee_denom: u64 = 10000;
    let amount_in_without_fee = amount_in
        .checked_mul(fee_numer)
        .and_then(|v| v.checked_div(fee_denom))
        .ok_or(ErrorCode::Overflow)?;
    let fee_amount = amount_in
        .checked_sub(amount_in_without_fee)
        .ok_or(ErrorCode::Overflow)?;
    msg!(
        "Cálculo do fee: amount_in = {}, amount_in_without_fee = {}, fee_amount = {}",
        amount_in,
        amount_in_without_fee,
        fee_amount
    );

    if liquidity == 0 {
        msg!("Erro: Liquidez zero.");
        return Err(ErrorCode::ZeroLiquidity.into());
    }

    let shift = 64u32;
    if zero_for_one {
        let liquidity_u256 = U256::from(liquidity);
        let sqrt_price_current_u256 = U256::from(sqrt_price_current);
        let amount_in_without_fee_u256 = U256::from(amount_in_without_fee);

        // numerator = (liquidity << shift) * sqrt_price_current
        let numerator = (liquidity_u256 << shift)
            .checked_mul(sqrt_price_current_u256)
            .ok_or(ErrorCode::Overflow)?;
        // denominator = (liquidity << shift) + (amount_in_without_fee * sqrt_price_current)
        let denominator = (liquidity_u256 << shift)
            .checked_add(
                amount_in_without_fee_u256
                    .checked_mul(sqrt_price_current_u256)
                    .ok_or(ErrorCode::Overflow)?,
            )
            .ok_or(ErrorCode::Overflow)?;
        let sqrt_p_next_u256 = numerator
            .checked_div(denominator)
            .ok_or(ErrorCode::Overflow)?;
        let sqrt_p_next = sqrt_p_next_u256.as_u128();
        msg!(
            "Zero_for_one: numerator = {}, denominator = {}, sqrtP_next = {}",
            numerator,
            denominator,
            sqrt_p_next
        );

        let diff = sqrt_price_current
            .checked_sub(sqrt_p_next)
            .ok_or(ErrorCode::Overflow)?;
        let amount_out = liquidity.checked_mul(diff).ok_or(ErrorCode::Overflow)? >> shift;
        Ok((sqrt_p_next, amount_in, amount_out as u64, fee_amount))
    } else {
        let sqrt_price_current_u256 = U256::from(sqrt_price_current);
        let liquidity_u256 = U256::from(liquidity);
        let amount_in_without_fee_u256 = U256::from(amount_in_without_fee);

        let increment_u256 = (amount_in_without_fee_u256 << shift)
            .checked_div(liquidity_u256)
            .ok_or(ErrorCode::Overflow)?;
        let sqrt_p_next_u256 = sqrt_price_current_u256
            .checked_add(increment_u256)
            .ok_or(ErrorCode::Overflow)?;
        let sqrt_p_next = sqrt_p_next_u256.as_u128();
        msg!(
            "One_for_zero: increment = {}, sqrtP_next = {}",
            increment_u256,
            sqrt_p_next_u256
        );

        let diff = sqrt_p_next
            .checked_sub(sqrt_price_current)
            .ok_or(ErrorCode::Overflow)?;
        let amount_out = liquidity.checked_mul(diff).ok_or(ErrorCode::Overflow)? >> shift;
        Ok((sqrt_p_next, amount_in, amount_out as u64, fee_amount))
    }
}
