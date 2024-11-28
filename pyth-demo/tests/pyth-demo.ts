import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PythDemo } from "../target/types/pyth_demo";
import { PublicKey } from "@solana/web3.js";
import { PriceServiceConnection } from "@pythnetwork/price-service-client";
import {
  InstructionWithEphemeralSigners,
  PythSolanaReceiver,
} from "@pythnetwork/pyth-solana-receiver";

const SOL_PRICE_FEED_ID =
  "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

const HERMES_URL = "https://hermes.pyth.network/";
const DEVNET_RPC_URL = "https://api.devnet.solana.com";

describe("pyth-demo", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet;
  const connection = provider.connection;
  const program = anchor.workspace.PythDemo as Program<PythDemo>;

  it("Pyth setup", async () => {
    const priceServiceConnection = new PriceServiceConnection(HERMES_URL, {
      priceFeedRequestConfig: { binary: true },
    });
    const pythSolanaReceiver = new PythSolanaReceiver({
      connection,
      wallet: wallet as anchor.Wallet,
    });

    const priceUpdateData = await priceServiceConnection.getLatestVaas([
      SOL_PRICE_FEED_ID,
    ]);

    const transactionBuilder = pythSolanaReceiver.newTransactionBuilder({
      closeUpdateAccounts: true,
    });

    await transactionBuilder.addPostPriceUpdates([priceUpdateData[0]]);

    // console.log(
    //   "priceFeedIdToPriceUpdateAccount:",
    //   transactionBuilder.priceFeedIdToPriceUpdateAccount
    // );

    await transactionBuilder.addPriceConsumerInstructions(
      async (
        getPriceUpdateAccount: (priceFeedId: string) => PublicKey
      ): Promise<InstructionWithEphemeralSigners[]> => {
        const priceUpdate = getPriceUpdateAccount(SOL_PRICE_FEED_ID);
        console.log("price update:", priceUpdate.toBase58());
        return [
          {
            instruction: await program.methods
              .initialize()
              .accounts({
                priceUpdate,
              })
              .instruction(),
            signers: [],
          },
        ];
      }
    );

    const txSigs = await pythSolanaReceiver.provider.sendAll(
      await transactionBuilder.buildVersionedTransactions({
        computeUnitPriceMicroLamports: 50000,
      }),
      { skipPreflight: true }
    );
    console.log("txSigs:", txSigs);
  });
});
