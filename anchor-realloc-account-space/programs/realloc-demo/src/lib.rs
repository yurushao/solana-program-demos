use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("FXFWZVRVm76Dw59UtcEGVWqno5C4U379sdo1PF423Z8A");

#[program]
mod realloc_demo {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.data.list = Vec::new();

        msg!(
            "Data account: {}",
            ctx.accounts.data.to_account_info().key()
        );

        return Ok(());
    }

    pub fn add(ctx: Context<Add>, entry: Pubkey) -> Result<()> {
        msg!("Adding new entry to vector: {}", entry);
        let data_account_info = ctx.accounts.data.to_account_info();

        let curr_data_size = data_account_info.data_len();
        let space_left = curr_data_size - ctx.accounts.data.list.len() * 32 - Data::INIT_SIZE;

        msg!("current data size: {}", curr_data_size);
        msg!("current length of list: {}", ctx.accounts.data.list.len());
        msg!("space left: {}", space_left);

        if space_left < 32 {
            let needed_len = curr_data_size + 10 * 32;
            AccountInfo::realloc(&data_account_info, needed_len, false)?;

            // if more lamports are needed, transfer them to the account
            let rent_exempt_lamports =
                ctx.accounts.rent.minimum_balance(needed_len).max(1);
            let top_up_lamports =
                rent_exempt_lamports.saturating_sub(ctx.accounts.data.to_account_info().lamports());

            msg!("top up lamports: {}", top_up_lamports);

            if top_up_lamports > 0 {
                anchor_lang::system_program::transfer(
                    anchor_lang::context::CpiContext::new(
                        ctx.accounts.system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: ctx.accounts.signer.to_account_info(),
                            to: ctx.accounts.data.to_account_info(),
                        },
                    ),
                    top_up_lamports
                )?;
            }
        }

        let curr_data_size = data_account_info.data_len();
        msg!("current data size after realloc: {}", curr_data_size);

        ctx.accounts.data.reload()?;
        ctx.accounts.data.list.push(entry);

        return Ok(());
    }
}

#[derive(Accounts)]
pub struct List<'info> {
    #[account()]
    pub data: Account<'info, Data>,
}

#[derive(Accounts)]
pub struct Add<'info> {
    #[account(mut)]
    pub data: Account<'info, Data>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        seeds = [b"data".as_ref(), signer.key().as_ref()], bump,
        payer = signer,
        space = Data::INIT_SIZE
    )]
    pub data: Account<'info, Data>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Data {
    pub list: Vec<Pubkey>,
}
impl Data {
    pub const INIT_SIZE: usize = 8 + 4;
}
