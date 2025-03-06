import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FaceSwap } from "../target/types/face_swap";
import { assert } from "chai";

describe("amm_clmm robust - CLMM com cálculos reais - Logs Detalhados", () => {
  // Configura o provider para cluster local.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Usa o programa FaceSwap conforme o IDL.
  const program = anchor.workspace.FaceSwap as Program<FaceSwap>;

  // Geradores de chaves para as contas do pool, tick accountsStrict e posição.
  const pool = anchor.web3.Keypair.generate();
  const lowerTickAccount = anchor.web3.Keypair.generate();
  const upperTickAccount = anchor.web3.Keypair.generate();
  const position = anchor.web3.Keypair.generate();

  // Função auxiliar para formatar BN para string legível.
  function formatBN(bn: anchor.BN): string {
    return bn.toString();
  }

  it("Inicializa o pool com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> INICIALIZAÇÃO DO POOL");
    console.log("============================================");

    // Define o preço inicial: 1 em Q64.64 (1 << 64)
    const sqrtPriceX64 = new anchor.BN(1).shln(64);
    const currentTick = 0;
    console.log("Parâmetros para inicialização do pool:");
    console.table({
      sqrtPriceX64: sqrtPriceX64.toString(),
      currentTick,
    });

    // Chama a instrução de inicialização do pool.
    const tx = await program.methods
      .initializePool(sqrtPriceX64, currentTick)
      .accountsStrict({
        pool: pool.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([pool])
      .rpc();
    console.log("Transação de inicialização do pool enviada. Tx:", tx);

    // Recupera o estado do pool.
    const poolAccount = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do pool após inicialização:");
    console.table({
      sqrtPriceX64: poolAccount.sqrtPriceX64.toString(),
      currentTick: poolAccount.currentTick,
      liquidity: poolAccount.liquidity.toString(),
    });

    // Verificações com assert.
    assert.ok(
      poolAccount.sqrtPriceX64.eq(sqrtPriceX64),
      "sqrtPriceX64 incorreto"
    );
    assert.ok(poolAccount.currentTick === currentTick, "currentTick incorreto");
    assert.ok(
      poolAccount.liquidity.eq(new anchor.BN(0)),
      "Liquidez inicial deve ser 0"
    );
    console.log(">> POOL inicializado com sucesso.\n");
  });

  it("Adiciona liquidez com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> ADIÇÃO DE LIQUIDEZ");
    console.log("============================================");

    // Inicializa o tick account para lowerTick (-10).
    console.log("Inicializando tick account para lowerTick (-10)...");
    const initLowerTx = await program.methods
      .initializeTick(-10)
      .accountsStrict({
        tick: lowerTickAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([lowerTickAccount])
      .rpc();
    console.log("Tick lower inicializado. Tx:", initLowerTx);

    // Inicializa o tick account para upperTick (10).
    console.log("Inicializando tick account para upperTick (10)...");
    const initUpperTx = await program.methods
      .initializeTick(10)
      .accountsStrict({
        tick: upperTickAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([upperTickAccount])
      .rpc();
    console.log("Tick upper inicializado. Tx:", initUpperTx);

    // Exibe o estado dos tick accountsStrict.
    const lowerTick = await program.account.tick.fetch(
      lowerTickAccount.publicKey
    );
    const upperTick = await program.account.tick.fetch(
      upperTickAccount.publicKey
    );
    console.log("Estado dos Tick accountsStrict após inicialização:");
    console.table({
      lowerTick: {
        tickIndex: lowerTick.tickIndex,
        liquidityNet: lowerTick.liquidityNet.toString(),
      },
      upperTick: {
        tickIndex: upperTick.tickIndex,
        liquidityNet: upperTick.liquidityNet.toString(),
      },
    });

    // Define os parâmetros para adicionar liquidez.
    const liquidityDelta = new anchor.BN(1000);
    const lowerTickValue = -10;
    const upperTickValue = 10;
    console.log("Parâmetros para addLiquidity:");
    console.table({
      liquidityDelta: liquidityDelta.toString(),
      lowerTick: lowerTickValue,
      upperTick: upperTickValue,
    });

    // Chama a instrução addLiquidity.
    const tx = await program.methods
      .addLiquidity(liquidityDelta, lowerTickValue, upperTickValue)
      .accountsStrict({
        pool: pool.publicKey,
        position: position.publicKey,
        lowerTickAccount: lowerTickAccount.publicKey,
        upperTickAccount: upperTickAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([position])
      .rpc();
    console.log("Transação de addLiquidity enviada. Tx:", tx);

    // Recupera o estado do pool após adicionar liquidez.
    const poolAccount = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do Pool após addLiquidity:");
    console.table({
      sqrtPriceX64: poolAccount.sqrtPriceX64.toString(),
      currentTick: poolAccount.currentTick,
      liquidity: poolAccount.liquidity.toString(),
    });

    // Validação: como o tick atual (0) está entre -10 e 10, a liquidez do pool deve ser igual a liquidityDelta.
    assert.ok(
      poolAccount.liquidity.eq(liquidityDelta),
      "Liquidez do pool não foi atualizada corretamente"
    );
    console.log(">> Liquidez adicionada com sucesso.\n");
  });

  it("Executa swap robusto (zero_for_one) com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> EXECUÇÃO DE SWAP (zero_for_one)");
    console.log("============================================");

    // Recupera o estado do pool antes do swap.
    const poolBefore = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do Pool antes do swap (zero_for_one):");
    console.table({
      sqrtPriceX64: poolBefore.sqrtPriceX64.toString(),
      liquidity: poolBefore.liquidity.toString(),
    });

    // Define os parâmetros do swap.
    const amountIn = new anchor.BN(500);
    const zeroForOne = true;
    console.log("Parâmetros do swap:");
    console.table({
      amountIn: amountIn.toString(),
      zeroForOne,
    });

    // Chama a instrução de swap.
    const tx = await program.methods
      .swap(amountIn, zeroForOne)
      .accountsStrict({
        pool: pool.publicKey,
        user: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Transação de swap (zero_for_one) enviada. Tx:", tx);

    // Recupera o estado do pool após o swap.
    const poolAfter = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do Pool após swap (zero_for_one):");
    console.table({
      sqrtPriceX64: poolAfter.sqrtPriceX64.toString(),
      liquidity: poolAfter.liquidity.toString(),
    });
    // Validação: o preço deve ter diminuído.
    assert.ok(
      poolAfter.sqrtPriceX64.lt(poolBefore.sqrtPriceX64),
      "O preço não diminuiu conforme esperado no swap zero_for_one"
    );
    console.log(">> Swap (zero_for_one) executado com sucesso.\n");
  });

  it("Executa swap robusto (one_for_zero) com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> EXECUÇÃO DE SWAP (one_for_zero)");
    console.log("============================================");

    // Recupera o estado do pool antes do swap.
    const poolBefore = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do Pool antes do swap (one_for_zero):");
    console.table({
      sqrtPriceX64: poolBefore.sqrtPriceX64.toString(),
      liquidity: poolBefore.liquidity.toString(),
    });

    // Define os parâmetros do swap.
    const amountIn = new anchor.BN(500);
    const oneForZero = false;
    console.log("Parâmetros do swap:");
    console.table({
      amountIn: amountIn.toString(),
      oneForZero,
    });

    // Chama a instrução de swap.
    const tx = await program.methods
      .swap(amountIn, oneForZero)
      .accountsStrict({
        pool: pool.publicKey,
        user: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Transação de swap (one_for_zero) enviada. Tx:", tx);

    // Recupera o estado do pool após o swap.
    const poolAfter = await program.account.pool.fetch(pool.publicKey);
    console.log("Estado do Pool após swap (one_for_zero):");
    console.table({
      sqrtPriceX64: poolAfter.sqrtPriceX64.toString(),
      liquidity: poolAfter.liquidity.toString(),
    });
    // Validação: o preço deve ter aumentado.
    assert.ok(
      poolAfter.sqrtPriceX64.gt(poolBefore.sqrtPriceX64),
      "O preço não aumentou conforme esperado no swap one_for_zero"
    );
    console.log(">> Swap (one_for_zero) executado com sucesso.\n");
  });
});
