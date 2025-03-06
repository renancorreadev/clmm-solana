# **Solana CLMM - Concentrated Liquidity Market Maker**

🚀 Implementação de um **CLMM (Concentrated Liquidity Market Maker)** na Solana usando **Anchor**. Este projeto permite a criação de pools de liquidez com faixas de preço definidas, oferecendo uma **eficiência maior no uso de capital** em comparação com os AMMs tradicionais.

---

## **📖 O que é um CLMM?**

Em um **AMM (Automated Market Maker) tradicional**, como **Uniswap V2**, os provedores de liquidez depositam tokens **em toda a faixa de preços**, tornando o uso do capital **ineficiente**.

Já no **CLMM (Concentrated Liquidity Market Maker)**:

- A liquidez é **fornecida dentro de uma faixa específica de preço**.
- Essa faixa é dividida em **ticks**, permitindo um **controle granular da liquidez**.
- Se o preço do ativo sair da faixa, **a liquidez fica inativa** até que o preço volte para dentro da faixa.

🛠 **Benefícios do CLMM:**
✅ Uso mais eficiente do capital.  
✅ Melhor controle sobre a liquidez.  
✅ Maior retorno para provedores de liquidez (LPs).

## **⚡ Funcionalidades Implementadas**

- **📌 Inicialização de Pool**

  - Cria um pool de liquidez com um preço inicial.
  - Define o **tick atual** e **o preço inicial** no formato `Q64.64`.

- **📌 Inicialização de Ticks**

  - Cada tick representa um **limite de preço** onde a liquidez pode ser ativada.
  - `lowerTickAccount` e `upperTickAccount` armazenam os ticks.

- **📌 Adição de Liquidez**

  - Um LP adiciona liquidez dentro de uma faixa (`lowerTick`, `upperTick`).
  - Se o **preço do ativo estiver dentro dessa faixa**, a liquidez será usada nos swaps.

- **📌 Swap de Tokens**

  - Permite trocas entre tokens usando **concentrated liquidity**.
  - O swap segue as regras do **Uniswap V3**, ajustando a liquidez e os ticks.

- **📌 Fee Collector**
  - Uma conta separada acumula **as taxas de swap**.
  - Permite rastrear os ganhos do pool de forma eficiente.

---

## **📌 Estrutura do Projeto**

```

```

└── 📁src
└── 📁contexts
└── mod.rs
└── pool_contexts.rs
└── 📁errors
└── code.rs
└── mod.rs
└── 📁instructions
└── initialize.rs
└── liquidity.rs
└── mod.rs
└── swap.rs
└── 📁state
└── fee_collector.rs
└── mod.rs
└── pool.rs
└── 📁utils
└── math.rs
└── mod.rs
└── lib.rs
└── mod.rs

````

---

## **⚙️ Como Rodar o Projeto**

### **1️⃣ Instalar Dependências**

```sh
yarn install
````

### **2️⃣ Compilar o Programa**

```sh
anchor build
```

### **3️⃣ Implantar na Localnet**

```sh
anchor deploy
```

### **4️⃣ Executar Testes**

```sh
anchor test
```

---

## **📜 Explicação dos Arquivos**

### 📌 **1. `lib.rs` - Entrada do Programa**

```rust
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
```

🎯 **Este arquivo define todas as funções do programa CLMM.**

---

### 📌 **2. `liquidity.rs` - Adição de Liquidez**

```rust
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

    Ok(())
}
```

🎯 **Esta função permite que LPs adicionem liquidez a uma faixa de ticks.**

---

### 📌 **3. `swap.rs` - Troca de Tokens**

```rust
pub fn swap(ctx: Context<Swap>, amount_in: u64, zero_for_one: bool) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let fee_collector = &mut ctx.accounts.fee_collector;

    let (new_sqrt_price, _, amount_out, fee_amount) =
        compute_swap_step(pool.sqrt_price_x64, pool.liquidity, amount_in, zero_for_one)?;

    pool.sqrt_price_x64 = new_sqrt_price;
    fee_collector.fees = fee_collector.fees.checked_add(fee_amount).ok_or(ErrorCode::Overflow)?;

    Ok(())
}
```

🎯 **Este código processa um swap e atualiza a liquidez do pool.**

---

## **📚 Conceitos Importantes**

### 🔹 **Ticks**

Ticks representam **faixas de preço discretas** onde a liquidez pode ser alocada.

```plaintext
|--- Tick -10 (R$4,80) ---|--- Tick 0 (R$5,00) ---|--- Tick 10 (R$5,20) ---|
```

Um **LP pode adicionar liquidez apenas dentro de um intervalo de ticks**.

### 🔹 **Q64.64**

Os preços no CLMM são armazenados no formato **Q64.64**, permitindo maior precisão matemática.

---

## **🚀 Conclusão**

```

Este projeto implementa um **CLMM na Solana** com suporte para **liquidez concentrada, swaps eficientes e um sistema de fee collector**. Ele permite **maior eficiência e melhor uso do capital** em comparação com os AMMs tradicionais.

```
