use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use drift::cpi::accounts::{
    CancelOrder, Deposit, InitializeUser, InitializeUserStats, PlaceOrders,
};
use drift::cpi::{cancel_order, deposit, initialize_user, initialize_user_stats, place_orders};
use drift::program::Drift;
use drift::OrderParams;

declare_id!("3eqvCf482d2Z1C7DevfrQZpzmKUh687kt7yhgLYUaYzu");

#[program]
pub mod drift_demo {

    use super::*;

    pub fn drift_initialize(ctx: Context<DriftInitialize>) -> Result<()> {
        initialize_user_stats(CpiContext::new(
            ctx.accounts.drift_program.to_account_info(),
            InitializeUserStats {
                user_stats: ctx.accounts.user_stats.to_account_info(),
                state: ctx.accounts.state.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
        ))?;

        let name = {
            let mut array = [0u8; 32];
            array[..8].copy_from_slice(b"GLAM *.+");
            array
        };
        initialize_user(
            CpiContext::new(
                ctx.accounts.drift_program.to_account_info(),
                InitializeUser {
                    user: ctx.accounts.user.to_account_info(),
                    user_stats: ctx.accounts.user_stats.to_account_info(),
                    state: ctx.accounts.state.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    payer: ctx.accounts.signer.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
            ),
            0,
            name,
        )?;

        Ok(())
    }

    pub fn drift_deposit<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, DriftDeposit<'info>>,
        market_index: u16,
        amount: u64,
    ) -> Result<()> {
        deposit(
            CpiContext::new(
                ctx.accounts.drift_program.to_account_info(),
                Deposit {
                    user: ctx.accounts.user.to_account_info(),
                    user_stats: ctx.accounts.user_stats.to_account_info(),
                    state: ctx.accounts.state.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    spot_market_vault: ctx.accounts.spot_market_vault.to_account_info(),
                    user_token_account: ctx.accounts.signer_wsol_ata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            )
            .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
            market_index,
            amount,
            false, // reduce_only
        )?;

        Ok(())
    }

    pub fn drift_place_orders<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, DriftPlaceOrder<'info>>,
        order_params: OrderParams,
    ) -> Result<()> {
        place_orders(
            CpiContext::new(
                ctx.accounts.drift_program.to_account_info(),
                PlaceOrders {
                    user: ctx.accounts.user.to_account_info(),
                    state: ctx.accounts.state.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            )
            .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
            [order_params].to_vec(),
        )?;

        Ok(())
    }

    pub fn drift_cancel_order<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, DriftCancelOrder<'info>>,
        order_id: Option<u32>,
    ) -> Result<()> {
        cancel_order(
            CpiContext::new(
                ctx.accounts.drift_program.to_account_info(),
                CancelOrder {
                    user: ctx.accounts.user.to_account_info(),
                    state: ctx.accounts.state.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            )
            .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
            order_id,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct DriftInitialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub signer_wsol_ata: Account<'info, TokenAccount>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user_stats: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub state: UncheckedAccount<'info>,

    pub drift_program: Program<'info, Drift>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DriftDeposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub signer_wsol_ata: Account<'info, TokenAccount>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user_stats: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub state: UncheckedAccount<'info>,

    #[account(mut)]
    pub spot_market_vault: Account<'info, TokenAccount>, // drift vault wsol ata

    pub token_program: Program<'info, Token>,
    pub drift_program: Program<'info, Drift>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DriftPlaceOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub state: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub drift_program: Program<'info, Drift>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DriftCancelOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// CHECK: checks are done inside cpi call
    #[account(mut)]
    pub state: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub drift_program: Program<'info, Drift>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
