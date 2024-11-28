import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DriftDemo } from "../target/types/drift_demo";
import {
  PublicKey,
  SystemProgram,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccountInstruction,
  createSyncNativeInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  BulkAccountLoader,
  DRIFT_PROGRAM_ID,
  DriftClient,
  getDriftStateAccountPublicKey,
  getOrderParams,
  getUserAccountPublicKeySync,
  getUserStatsAccountPublicKey,
  initialize,
  MarketType,
  OrderType,
  PerpMarkets,
  PositionDirection,
  SpotMarkets,
} from "@drift-labs/sdk";
import { Connection } from "@solana/web3.js";

const WSOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");

async function wrapSOL(wallet: anchor.Wallet, connection: Connection) {
  // Get the WSOL ATA for the wallet
  const wsolAta = getAssociatedTokenAddressSync(WSOL_MINT, wallet.publicKey);
  let instructions = [];

  // If the ATA doesn't exist, create the ATA
  const accountInfo = await connection.getAccountInfo(wsolAta);
  if (!accountInfo) {
    const createAtaInstruction = createAssociatedTokenAccountInstruction(
      wallet.publicKey, // payer
      wsolAta, // address of the ATA
      wallet.publicKey, // owner of the ATA
      WSOL_MINT // WSOL mint
    );
    instructions.push(createAtaInstruction);
  }

  // Transfer SOL to the WSOL ATA (effectively wrapping the SOL)
  const transferInstruction = SystemProgram.transfer({
    fromPubkey: wallet.publicKey,
    toPubkey: wsolAta,
    lamports: 10_000_000_000,
  });

  instructions.push(transferInstruction);

  // Sync native instruction to update the WSOL balance
  const syncInstruction = createSyncNativeInstruction(wsolAta);
  instructions.push(syncInstruction);

  const latestBlockhash = await connection.getLatestBlockhash();
  const messageV0 = new TransactionMessage({
    payerKey: wallet.publicKey,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: instructions,
  }).compileToV0Message();
  const vTx = new VersionedTransaction(messageV0);

  const signedTx = await wallet.signTransaction(vTx);
  const serializedTx = signedTx.serialize();
  const signature = await connection.sendRawTransaction(serializedTx, {
    skipPreflight: true,
  });

  console.log("Transaction signature:", signature);

  const res = await connection.confirmTransaction({
    ...latestBlockhash,
    signature,
  });

  console.log("Transaction confirmed:", res);
}

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;

describe("drift_cpi_demo", () => {
  console.log("Local wallet address", wallet.publicKey.toBase58());

  const program = anchor.workspace.DriftDemo as Program<DriftDemo>;

  const wsolAta = getAssociatedTokenAddressSync(WSOL_MINT, wallet.publicKey);
  let user, userStats, state;

  before(async () => {
    await wrapSOL(wallet as anchor.Wallet, provider.connection);

    user = getUserAccountPublicKeySync(
      new PublicKey(DRIFT_PROGRAM_ID),
      wallet.publicKey
    );

    userStats = await getUserStatsAccountPublicKey(
      new PublicKey(DRIFT_PROGRAM_ID),
      wallet.publicKey
    );

    state = await getDriftStateAccountPublicKey(
      new PublicKey(DRIFT_PROGRAM_ID)
    );

    console.log("Signer wsol ata:", wsolAta.toBase58());
    console.log("User account:", user.toBase58());
    console.log("User stats account:", userStats.toBase58());
    console.log("State account:", state.toBase58());
  });

  it("Initialize drift setup", async () => {
    const tx = await program.methods
      .driftInitialize()
      .accounts({
        signer: wallet.publicKey,
        signerWsolAta: wsolAta,
        user,
        userStats,
        state,
      })
      .rpc({ commitment: "confirmed" });
    console.log("driftInitialize txSig:", tx);
  });

  it("Deposit 1 SOL to drift", async () => {
    const tx = await program.methods
      .driftDeposit(1, new anchor.BN(1_000_000_000))
      .accounts({
        signer: wallet.publicKey,
        signerWsolAta: wsolAta,
        user,
        userStats,
        state,
        spotMarketVault: new PublicKey(
          "DfYCNezifxAEsQbAJ1b3j6PX3JVBe8fu11KBhxsbw5d2"
        ),
      })
      .remainingAccounts([
        {
          // oracle
          pubkey: new PublicKey("HpMoKp3TCd3QT4MWYUKk2zCBwmhr5Df45fB6wdxYqEeh"),
          isWritable: false,
          isSigner: false,
        },
        {
          // oracle
          pubkey: new PublicKey("BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF"),
          isWritable: false,
          isSigner: false,
        },
        {
          // perp market?
          pubkey: new PublicKey("GyyHYVCrZGc2AQPuvNbcP1babmU3L42ptmxZthUfD9q"),
          isWritable: false,
          isSigner: false,
        },
        {
          // Drift (SOL) Spot Market
          pubkey: new PublicKey("3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh"),
          isWritable: true,
          isSigner: false,
        },
      ])
      .rpc({ commitment: "confirmed" });
    console.log("driftDeposit txSig:", tx);
  });

  it("Place perp order", async () => {
    const orderParams = getOrderParams({
      orderType: OrderType.LIMIT,
      marketType: MarketType.PERP,
      direction: PositionDirection.LONG,
      marketIndex: 0,
      baseAssetAmount: new anchor.BN(10_0000_000),
      price: new anchor.BN(156_597_000),
    });
    console.log("Spot market 0:", SpotMarkets["mainnet-beta"][0]);
    console.log("orderParams", orderParams);

    const tx = await program.methods
      .driftPlaceOrders(orderParams)
      .accounts({
        signer: wallet.publicKey,
        user,
        state,
      })
      .remainingAccounts([
        {
          pubkey: new PublicKey("HpMoKp3TCd3QT4MWYUKk2zCBwmhr5Df45fB6wdxYqEeh"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF"),
          isWritable: false,
          isSigner: false,
        },

        {
          pubkey: new PublicKey("En8hkHLkRe9d9DraYmBTrus518BvmVH448YcvmrFM6Ce"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("GyyHYVCrZGc2AQPuvNbcP1babmU3L42ptmxZthUfD9q"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh"),
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("6gMq3mRCKf8aP3ttTyYhuijVZ2LGi14oDsBbkgubfLB3"),
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("8UJgxaiQx5nTrdDgph5FiahMmzduuLTLf5WmsPegYA6W"),
          isWritable: true,
          isSigner: false,
        },
      ])
      .rpc({ commitment: "confirmed" });
    console.log("driftPlaceOrders txSig:", tx);
  });

  it("Cancel order", async () => {
    const tx = await program.methods
      .driftCancelOrder(1) // order id starts from 1
      .accounts({
        signer: wallet.publicKey,
        user,
        state,
      })
      .remainingAccounts([
        {
          pubkey: new PublicKey("HpMoKp3TCd3QT4MWYUKk2zCBwmhr5Df45fB6wdxYqEeh"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF"),
          isWritable: false,
          isSigner: false,
        },

        {
          pubkey: new PublicKey("En8hkHLkRe9d9DraYmBTrus518BvmVH448YcvmrFM6Ce"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("GyyHYVCrZGc2AQPuvNbcP1babmU3L42ptmxZthUfD9q"),
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh"),
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("6gMq3mRCKf8aP3ttTyYhuijVZ2LGi14oDsBbkgubfLB3"),
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: new PublicKey("8UJgxaiQx5nTrdDgph5FiahMmzduuLTLf5WmsPegYA6W"),
          isWritable: true,
          isSigner: false,
        },
      ])
      .rpc({ commitment: "confirmed" });
    console.log("driftCancelOrder txSig:", tx);
  });
});
