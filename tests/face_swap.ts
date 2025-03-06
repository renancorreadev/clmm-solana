import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FaceSwap } from "../target/types/face_swap";
import { assert } from "chai";

describe("amm_clmm robust - CLMM ", () => {
  // Configura o provider para cluster local.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // uses the FaceSwap program as per the IDL.
  const program = anchor.workspace.FaceSwap as Program<FaceSwap>;

  // generato of keys for the pool, ticks and position accounts.
  const pool = anchor.web3.Keypair.generate();
  const lowerTickAccount = anchor.web3.Keypair.generate();
  const upperTickAccount = anchor.web3.Keypair.generate();
  const position = anchor.web3.Keypair.generate();

  // the account of the fee collector derived from the seed "fee_collector"
  let feeCollectorPda: anchor.web3.PublicKey;

  it("Inicializa o pool com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> INICIALIZAÇÃO DO POOL");
    console.log("============================================");

    const sqrtPriceX64 = new anchor.BN(1).shln(64);
    const currentTick = 0;
    console.log("Parâmetros para inicialização do pool:");
    console.table({ sqrtPriceX64: sqrtPriceX64.toString(), currentTick });

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
  });

  it("Inicializa a conta feeCollector corretamente", async () => {
    console.log("============================================");
    console.log(">> INICIALIZAÇÃO DO FEE COLLECTOR");
    console.log("============================================");

    // derives the PDA for feeCollector with the seed "fee_collector".
    const [pda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("fee_collector")],
      program.programId
    );
    feeCollectorPda = pda;

    const tx = await program.methods
      .initializeFeeCollector()
      .accountsStrict({
        feeCollector: feeCollectorPda,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("FeeCollector inicializado com sucesso. Tx:", tx);

    // verifies if the account was initialized correctly
    const feeCollectorAccount = await program.account.feeCollector.fetch(
      feeCollectorPda
    );
    assert.ok(
      feeCollectorAccount.fees.eq(new anchor.BN(0)),
      "FeeCollector deve iniciar com 0"
    );
  });

  it("Adiciona liquidez ao pool", async () => {
    console.log("============================================");
    console.log(">> ADIÇÃO DE LIQUIDEZ");
    console.log("============================================");

    // definition of the ticks of the liquidity position
    const lowerTick = -10;
    const upperTick = 10;
    const liquidityDelta = new anchor.BN(1000);

    console.log("Inicializando tick accounts...");

    // initializes the lower tick
    await program.methods
      .initializeTick(lowerTick)
      .accountsStrict({
        tick: lowerTickAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([lowerTickAccount])
      .rpc();

    // initializes the upper tick
    await program.methods
      .initializeTick(upperTick)
      .accountsStrict({
        tick: upperTickAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([upperTickAccount])
      .rpc();

    console.log("Parâmetros de liquidez:");
    console.table({
      liquidityDelta: liquidityDelta.toString(),
      lowerTick,
      upperTick,
    });

    // adds liquidity to the pool
    const tx = await program.methods
      .addLiquidity(liquidityDelta, lowerTick, upperTick)
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

    console.log("Liquidez adicionada com sucesso. Tx:", tx);
  });

  it("Executa swap robusto (zero_for_one) com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> EXECUÇÃO DE SWAP (zero_for_one)");
    console.log("============================================");

    const amountIn = new anchor.BN(500);
    console.log("Parâmetros do swap:");
    console.table({ amountIn: amountIn.toString(), zeroForOne: true });

    const tx = await program.methods
      .swap(amountIn, true)
      .accounts({
        pool: pool.publicKey,
        feeCollector: feeCollectorPda,
        user: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Transação de swap (zero_for_one) enviada. Tx:", tx);
  });

  it("Executa swap robusto (one_for_zero) com logs detalhados", async () => {
    console.log("============================================");
    console.log(">> EXECUÇÃO DE SWAP (one_for_zero)");
    console.log("============================================");

    const amountIn = new anchor.BN(500);
    console.log("Parâmetros do swap:");
    console.table({ amountIn: amountIn.toString(), oneForZero: false });

    const tx = await program.methods
      .swap(amountIn, false)
      .accounts({
        pool: pool.publicKey,
        feeCollector: feeCollectorPda,
        user: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Transação de swap (one_for_zero) enviada. Tx:", tx);
  });
});
