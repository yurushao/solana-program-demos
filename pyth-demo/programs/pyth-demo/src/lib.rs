use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

declare_id!("EZF96kTTwgT9EPGz1aAvvc9ZZ7r74Rv4tA4ARNKroaCE");

pub const MAXIMUM_AGE: u64 = 60; // One minute
pub const FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"; // SOL/USD price feed id from https://pyth.network/developers/price-feed-ids

#[program]
pub mod pyth_demo {
    use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let price_update = &mut ctx.accounts.price_update;
        let price = price_update.get_price_no_older_than(
            &Clock::get()?,
            MAXIMUM_AGE,
            &get_feed_id_from_hex(FEED_ID)?,
        )?;

        msg!(
            "The price is ({} ± {}) * 10^{}",
            price.price,
            price.conf,
            price.exponent
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // #[account(mut)]
    // pub payer: Signer<'info>,
    pub price_update: Account<'info, PriceUpdateV2>,
}
