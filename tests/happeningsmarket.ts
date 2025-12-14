import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Happeningsmarket } from "../target/types/happeningsmarket";

describe("happeningsmarket", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.happeningsmarket as Program<Happeningsmarket>;

  const adminWallet = program.provider.wallet;

  const userWallet = new anchor.web3.Keypair();

  it("initialize", async () => {
    let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),

        adminWallet.publicKey.toBuffer(),

        new anchor.BN(2).toArrayLike(Buffer, "le", 4),
      ],

      program.programId
    );

    const txHash = await program.methods

      .createMarket(2, 7)

      .accounts({
        market: market,

        signer: adminWallet.publicKey,

        systemProgram: anchor.web3.SystemProgram.programId,
      })

      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    await program.provider.connection.confirmTransaction(txHash);

    const newMarket = await program.account.market.fetch(market);

    console.log("On-chain data is:", newMarket);
  });

  it("Place Bet", async () => {
    let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),

        adminWallet.publicKey.toBuffer(),

        new anchor.BN(2).toArrayLike(Buffer, "le", 4),
      ],

      program.programId
    );

    const txHash = await program.methods

      .placeBet(3, true, false, 4)

      .accounts({
        market: market,

        signer: userWallet.publicKey,
      })

      .signers([userWallet])

      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    await program.provider.connection.confirmTransaction(txHash);

    const newMarket = await program.account.market.fetch(market);

    console.log("On-chain data is:", newMarket);
  });
});
