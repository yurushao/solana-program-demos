import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import {
  BulkAccountLoader,
  DRIFT_PROGRAM_ID,
  DriftClient,
  initialize,
  PerpMarkets,
  SpotMarkets,
} from "@drift-labs/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;

describe("drift_client_test", () => {
  it("test", async () => {
    const solPerpMarketInfo = PerpMarkets["mainnet-beta"].find(
      (market) => market.baseAssetSymbol === "SOL"
    );

    const solSpotMarketInfo = SpotMarkets["mainnet-beta"].find(
      (market) => market.symbol === "SOL"
    );

    // console.log("solPerpMarketInfo", solPerpMarketInfo);
    // console.log("solSpotMarketInfo", solSpotMarketInfo);

    const sdkConfig = initialize({ env: "mainnet-beta" });
    const bulkAccountLoader = new BulkAccountLoader(
      provider.connection,
      "confirmed",
      1000
    );
    const driftClient = new DriftClient({
      connection: provider.connection,
      wallet,
      programID: new PublicKey(DRIFT_PROGRAM_ID),
      env: "mainnet-beta",
      accountSubscription: {
        type: "polling",
        accountLoader: bulkAccountLoader,
      },
    });
    await driftClient.subscribe();

    console.log("subscribed to driftClient:", driftClient.isSubscribed);

    // const solSpotMarketAccount = driftClient.getSpotMarketAccount(
    // solSpotMarketInfo.marketIndex
    // );
    const solPerpMarketAccount = driftClient.getPerpMarketAccount(
      solPerpMarketInfo.marketIndex
    );
    // console.log("solSpotMarketAccount", solSpotMarketAccount);
    console.log("solPerpMarketAccount", solPerpMarketAccount);

    // console.log("spotMarketAccounts", driftClient.getSpotMarketAccounts());
  });
});
