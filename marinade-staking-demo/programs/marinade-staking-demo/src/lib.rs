use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use marinade::cpi::accounts::{
    Claim as MarinadeClaim, Deposit as MarinadeDeposit, LiquidUnstake as MarinadeLiquidUnstake,
    OrderUnstake as MarinadeOrderUnstake,
};
use marinade::cpi::{
    claim as marinade_claim, deposit as marinade_deposit,
    liquid_unstake as marinade_liquid_unstake, order_unstake as marinade_order_unstake,
};
use marinade::program::MarinadeFinance;
use marinade::State as MarinadeState;
use marinade::TicketAccountData;

declare_id!("DfHJpPQ7BNhGz9LNvohUgqFMDRqfLDeDXk8GKqnryNPT");

#[program]
pub mod marinade_staking_demo {
    use super::*;

    pub fn init<'c: 'info, 'info>(ctx: Context<Init>, treasury_pda_bump: u8) -> Result<()> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(500); // Minimum balance to make the account rent-exempt

        let seeds = &["treasury".as_bytes(), &[treasury_pda_bump]];
        let signer_seeds = &[&seeds[..]];

        system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.treasury_pda.to_account_info().clone(),
                },
                signer_seeds,
            ),
            lamports,
            0, // no data
            &ctx.accounts.system_program.key(),
        )?;

        Ok(())
    }

    pub fn deposit<'c: 'info, 'info>(
        ctx: Context<Deposit>,
        sol_amount: u64,
        treasury_bump: u8,
    ) -> Result<()> {
        msg!(
            "mSol will be mint to ATA: {:?} owned by {:?}",
            ctx.accounts.mint_to.key(),
            ctx.accounts.treasury_pda.key()
        );

        msg!(
            "transfer_from lamports: {:?}",
            ctx.accounts.treasury_pda.lamports()
        );

        require_gte!(ctx.accounts.treasury_pda.lamports(), sol_amount);

        let cpi_program = ctx.accounts.marinade_program.to_account_info();
        let cpi_accounts = MarinadeDeposit {
            state: ctx.accounts.marinade_state.to_account_info(),
            msol_mint: ctx.accounts.msol_mint.to_account_info(),
            liq_pool_sol_leg_pda: ctx.accounts.liq_pool_sol_leg_pda.to_account_info(),
            liq_pool_msol_leg: ctx.accounts.liq_pool_msol_leg.to_account_info(),
            liq_pool_msol_leg_authority: ctx.accounts.liq_pool_msol_leg_authority.to_account_info(),
            reserve_pda: ctx.accounts.reserve_pda.to_account_info(),
            transfer_from: ctx.accounts.treasury_pda.to_account_info(),
            mint_to: ctx.accounts.mint_to.to_account_info(),
            msol_mint_authority: ctx.accounts.msol_mint_authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let seeds = &[b"treasury".as_ref(), &[treasury_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        let _ = marinade_deposit(cpi_ctx, sol_amount);
        Ok(())
    }

    pub fn delayed_unstake<'c: 'info, 'info>(
        ctx: Context<DelayedUnstake>,
        msol_amount: u64,
        ticket_bump: u8,
        treasury_bump: u8,
    ) -> Result<()> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(500); // Minimum balance to make the account rent-exempt

        let seeds = &["ticket".as_bytes(), &[ticket_bump]];
        let signer_seeds = &[&seeds[..]];
        let space = std::mem::size_of::<TicketAccountData>() as u64 + 8;

        msg!(
            "Creating ticket account with address {}",
            ctx.accounts.ticket.key()
        );

        system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.signer.to_account_info(), // treasury PDA
                    to: ctx.accounts.ticket.to_account_info().clone(),
                },
                signer_seeds,
            ),
            lamports,
            space,
            &ctx.accounts.marinade_program.key(),
        )?;

        let cpi_program = ctx.accounts.marinade_program.to_account_info();
        let cpi_accounts = MarinadeOrderUnstake {
            state: ctx.accounts.marinade_state.to_account_info(),
            msol_mint: ctx.accounts.msol_mint.to_account_info(),
            burn_msol_from: ctx.accounts.burn_msol_from.to_account_info(),
            burn_msol_authority: ctx.accounts.burn_msol_authority.to_account_info(),
            new_ticket_account: ctx.accounts.ticket.to_account_info(),
            clock: ctx.accounts.clock.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let seeds = &[b"treasury".as_ref(), &[treasury_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        let _ = marinade_order_unstake(cpi_ctx, msol_amount);

        Ok(())
    }

    pub fn claim<'c: 'info, 'info>(ctx: Context<Claim>, treasury_bump: u8) -> Result<()> {
        let cpi_program = ctx.accounts.marinade_program.to_account_info();
        let cpi_accounts = MarinadeClaim {
            state: ctx.accounts.marinade_state.to_account_info(),
            ticket_account: ctx.accounts.ticket.to_account_info(),
            transfer_sol_to: ctx.accounts.transfer_sol_to.to_account_info(),
            reserve_pda: ctx.accounts.reserve_pda.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            clock: ctx.accounts.clock.to_account_info(),
        };
        let seeds = &[b"treasury".as_ref(), &[treasury_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        let _ = marinade_claim(cpi_ctx);
        Ok(())
    }

    pub fn unstake<'c: 'info, 'info>(
        ctx: Context<Unstake>,
        msol_amount: u64,
        treasury_bump: u8,
    ) -> Result<()> {
        let cpi_program = ctx.accounts.marinade_program.to_account_info();
        let cpi_accounts = MarinadeLiquidUnstake {
            state: ctx.accounts.marinade_state.to_account_info(),
            msol_mint: ctx.accounts.msol_mint.to_account_info(),
            liq_pool_sol_leg_pda: ctx.accounts.liq_pool_sol_leg_pda.to_account_info(),
            liq_pool_msol_leg: ctx.accounts.liq_pool_msol_leg.to_account_info(),
            get_msol_from: ctx.accounts.get_msol_from.to_account_info(),
            get_msol_from_authority: ctx.accounts.get_msol_from_authority.to_account_info(),
            transfer_sol_to: ctx.accounts.transfer_sol_to.to_account_info(),
            treasury_msol_account: ctx.accounts.treasury_msol_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let seeds = &[b"treasury".as_ref(), &[treasury_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        let _ = marinade_liquid_unstake(cpi_ctx, msol_amount);
        Ok(())
    }
}

#[account]
pub struct Treasury {}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // #[account(
    //     init_if_needed,
    //     seeds = [b"treasury".as_ref()],
    //     bump,
    //     payer = signer,
    //     owner = system_program.key(),
    //     space = 8
    // )]
    /// CHECK: will be initialized in the program
    #[account(mut)]
    pub treasury_pda: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub marinade_state: Account<'info, MarinadeState>,

    /// CHECK: skip
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK: skip
    #[account(mut)]
    pub msol_mint_authority: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = msol_mint,
        associated_token::authority = treasury_pda,
        associated_token::token_program = token_program
    )]
    pub mint_to: Account<'info, TokenAccount>,

    /// CHECK: skip
    #[account(mut)]
    pub treasury_pda: AccountInfo<'info>,

    pub marinade_program: Program<'info, MarinadeFinance>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DelayedUnstake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: skip
    // #[account(init_if_needed, seeds = [b"ticket"], bump, payer = signer, space = 88, owner = marinade_program.key())]
    #[account(mut)]
    pub ticket: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub msol_mint: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub burn_msol_from: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub burn_msol_authority: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub marinade_program: Program<'info, MarinadeFinance>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: skip
    // #[account(init_if_needed, seeds = [b"ticket"], bump, payer = signer, space = 88, owner = marinade_program.key())]
    #[account(mut)]
    pub ticket: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub marinade_program: Program<'info, MarinadeFinance>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    // pub signer: Signer<'info>,
    /// CHECK: skip
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub msol_mint: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub get_msol_from: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub get_msol_from_authority: AccountInfo<'info>,

    /// CHECK: skip
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    pub marinade_program: Program<'info, MarinadeFinance>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
