#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use primitive_types::U256;

declare_id!("56xKyGcHxr8VPv8mjfyaieMMUymqiEWFqf1tDSoKYcth");

pub mod contexts {
    use super::*;

    pub fn create_pool(
        ctx: Context<InitializePool>,
        sqrt_price_x64: u128,
        current_tick: i32,
    ) -> Result<()> {
        // Aqui já chamamos a instrução de inicialização do pool.
        face_swap::initialize_pool(ctx, sqrt_price_x64, current_tick)
    }

    /// Função helper para criar e inicializar um Tick.
    pub fn create_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
        face_swap::initialize_tick(ctx, tick_index)
    }
}

#[program]
pub mod face_swap {
    use super::*;

    /// Inicializa o pool com preço inicial, tick atual e liquidez zero.
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

    /// Inicializa uma conta Tick com um tick index e liquidez zero.
    pub fn initialize_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
        let tick = &mut ctx.accounts.tick;
        tick.tick_index = tick_index;
        tick.liquidity_net = 0;
        Ok(())
    }

    /// Adiciona liquidez ao pool, criando uma posição para o LP e atualizando os ticks.
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        liquidity_delta: u128,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<()> {
        if lower_tick >= upper_tick {
            return Err(ErrorCode::InvalidTickRange.into());
        }

        // Inicializa a posição do LP.
        let position = &mut ctx.accounts.position;
        position.owner = ctx.accounts.user.key();
        position.liquidity_delta = liquidity_delta;
        position.lower_tick = lower_tick;
        position.upper_tick = upper_tick;

        // Atualiza a liquidez global do pool se o tick atual estiver na faixa.
        let pool = &mut ctx.accounts.pool;
        if pool.current_tick >= lower_tick && pool.current_tick < upper_tick {
            pool.liquidity = pool
                .liquidity
                .checked_add(liquidity_delta)
                .ok_or(ErrorCode::Overflow)?;
        }

        // Atualiza os tick accounts.
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

    /// Realiza um swap robusto utilizando cálculos inspirados no Uniswap V3.
    /// O parâmetro `zero_for_one` indica a direção do swap:
    /// - true: troca token0 por token1;
    /// - false: troca token1 por token0.
    pub fn swap(ctx: Context<Swap>, amount_in: u64, zero_for_one: bool) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let (new_sqrt_price, amount_in_consumed, amount_out, fee_amount) =
            compute_swap_step(pool.sqrt_price_x64, pool.liquidity, amount_in, zero_for_one)?;

        // Atualiza o preço do pool (a lógica para cruzamento de ticks não está implementada)
        pool.sqrt_price_x64 = new_sqrt_price;

        msg!(
            "Swap: in_consumed={}, out={}, fee={}",
            amount_in_consumed,
            amount_out,
            fee_amount
        );
        Ok(())
    }
}

fn compute_swap_step(
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
        // Converte os valores para U256
        let liquidity_u256 = U256::from(liquidity);
        let sqrt_price_current_u256 = U256::from(sqrt_price_current);
        let amount_in_without_fee_u256 = U256::from(amount_in_without_fee);

        // Calcula:
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
        // Para one_for_zero, utiliza a abordagem similar.
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

/// Contexto para inicializar o pool.
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + Pool::LEN)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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
    const LEN: usize = 16 + 4 + 16;
}

/// Contexto para inicializar uma conta do tipo Tick.
#[derive(Accounts)]
#[instruction(tick_index: i32)]
pub struct InitializeTick<'info> {
    #[account(init, payer = user, space = 8 + Tick::LEN)]
    pub tick: Account<'info, Tick>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
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
    const LEN: usize = 4 + 16;
}

/// Contexto para adicionar liquidez.
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
    const LEN: usize = 32 + 16 + 4 + 4;
}

/// Contexto para executar o swap.
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
}

/// Lista de códigos de erro.
#[error_code]
pub enum ErrorCode {
    #[msg("Faixa de ticks inválida")]
    InvalidTickRange,
    #[msg("Overflow aritmético")]
    Overflow,
    #[msg("Liquidez zero")]
    ZeroLiquidity,
}
