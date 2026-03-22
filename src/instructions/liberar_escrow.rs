use anchor_lang::prelude::*;

use crate::errors::Errores;
use crate::state::escrow::{EscrowPda, ESCROW_LOCKED, ESCROW_RELEASED};

pub fn handler(ctx: Context<LiberarEscrow>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_pda;

    // Solo el buyer puede confirmar la entrega y liberar los fondos
    require!(
        ctx.accounts.buyer.key() == escrow.buyer,
        Errores::NoEresElBuyer
    );

    // El escrow debe estar en estado LOCKED para poder liberarse
    require!(escrow.status == ESCROW_LOCKED, Errores::EstadoInvalido);

    let amount = escrow.amount;

    // Transferir lamports de la PDA al seller manipulando los balances directamente.
    // Este es el patrón correcto en Anchor para PDAs que reciben SOL nativo.
    **escrow.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.seller.try_borrow_mut_lamports()? += amount;

    escrow.status = ESCROW_RELEASED;

    msg!(
        "Escrow liberado: {} | seller: {} | amount: {} lamports",
        escrow.transaction_id,
        escrow.seller,
        amount
    );
    Ok(())
}

#[derive(Accounts)]
pub struct LiberarEscrow<'info> {
    /// El comprador confirma la recepción del producto
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: wallet del vendedor — recibe los fondos
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"escrow", buyer.key().as_ref(), escrow_pda.transaction_id.as_bytes()],
        bump = escrow_pda.bump
    )]
    pub escrow_pda: Account<'info, EscrowPda>,
}
