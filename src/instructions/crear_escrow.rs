use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::Errores;
use crate::state::escrow::{EscrowPda, ESCROW_LOCKED};

pub fn handler(ctx: Context<CrearEscrow>, transaction_id: String, amount: u64) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_pda;

    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.amount = amount;
    escrow.status = ESCROW_LOCKED;
    escrow.transaction_id = transaction_id.clone();
    escrow.bump = ctx.bumps.escrow_pda;

    // Transferir los fondos del buyer a la PDA
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.escrow_pda.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!(
        "Escrow creado: {} | buyer: {} | seller: {} | amount: {} lamports",
        transaction_id,
        escrow.buyer,
        escrow.seller,
        amount
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(transaction_id: String)]
pub struct CrearEscrow<'info> {
    /// El comprador firma y paga la creación de la cuenta escrow
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: wallet del vendedor — solo se almacena en la PDA
    pub seller: AccountInfo<'info>,

    #[account(
        init,
        payer = buyer,
        space = EscrowPda::INIT_SPACE + 8,
        seeds = [b"escrow", buyer.key().as_ref(), transaction_id.as_bytes()],
        bump
    )]
    pub escrow_pda: Account<'info, EscrowPda>,

    pub system_program: Program<'info, System>,
}
