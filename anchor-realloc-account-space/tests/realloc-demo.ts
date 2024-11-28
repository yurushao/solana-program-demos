import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ReallocDemo } from "../target/types/realloc_demo";
import { PublicKey } from "@solana/web3.js";
import { SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { assert } from "chai";

describe("realloc-demo", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.ReallocDemo as Program<ReallocDemo>;

  const [dataPda, dataBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("data"), wallet.publicKey.toBuffer()],
    program.programId
  );
  console.log("dataPda", dataPda.toBase58());

  it("Is initialized!", async () => {
    const tx = await program.methods
      .initialize()
      .accounts({ data: dataPda, signer: wallet.publicKey })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Add data", async () => {
    try {
      await program.methods
        .add(dataPda)
        .accounts({
          data: dataPda,
          signer: wallet.publicKey,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      await program.methods
        .add(wallet.publicKey)
        .accounts({
          data: dataPda,
          signer: wallet.publicKey,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();
    } catch (e) {
      console.error(e);
      throw e;
    }

    const { list } = await program.account.data.fetch(dataPda);
    console.log("list", list);
    assert.equal(list.length, 2);
    assert.equal(list[0].toBase58(), dataPda.toBase58());
    assert.equal(list[1].toBase58(), wallet.publicKey.toBase58());
  });
});
