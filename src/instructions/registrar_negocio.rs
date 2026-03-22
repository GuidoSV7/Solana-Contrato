use anchor_lang::prelude::*;

use crate::state::business::BusinessPda;

pub fn handler(ctx: Context<RegistrarNegocio>, business_id: String) -> Result<()> {
    let business = &mut ctx.accounts.business_pda;

    business.owner = ctx.accounts.owner.key();
    business.business_id = business_id.clone();
    business.is_verified = false;
    business.bump = ctx.bumps.business_pda;

    msg!(
        "Negocio registrado: {} | wallet: {}",
        business_id,
        business.owner
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(business_id: String)]
pub struct RegistrarNegocio<'info> {
    /// Backend de TrustPay: firma y paga la creación de la cuenta
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: wallet del merchant — solo se usa como semilla de la PDA
    pub owner: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = BusinessPda::INIT_SPACE + 8,
        seeds = [b"business", owner.key().as_ref()],
        bump
    )]
    pub business_pda: Account<'info, BusinessPda>,

    pub system_program: Program<'info, System>,
}
