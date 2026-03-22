use anchor_lang::prelude::*;

use crate::errors::Errores;
use crate::state::business::BusinessPda;
use crate::TRUSTPAY_AUTHORITY;

pub fn handler(ctx: Context<VerificarNegocio>) -> Result<()> {
    let business = &mut ctx.accounts.business_pda;

    require!(!business.is_verified, Errores::YaVerificado);

    business.is_verified = true;

    msg!(
        "Negocio verificado: {} | wallet: {}",
        business.business_id,
        business.owner
    );
    Ok(())
}

#[derive(Accounts)]
pub struct VerificarNegocio<'info> {
    /// Solo la authority de TrustPay puede llamar esta instrucción
    #[account(
        mut,
        constraint = authority.key() == TRUSTPAY_AUTHORITY @ Errores::NoAutorizado
    )]
    pub authority: Signer<'info>,

    /// CHECK: wallet del merchant — solo se usa para derivar la PDA
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"business", owner.key().as_ref()],
        bump = business_pda.bump
    )]
    pub business_pda: Account<'info, BusinessPda>,
}
