use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

declare_id!("6R5q6P5975rhYg2KXaGLPjysYBtMHGwqBajmYDJLwVih");

// Pubkey del keypair del backend TrustPay (authority).
// Reemplazar con la clave real antes de compilar.
pub const TRUSTPAY_AUTHORITY: Pubkey = pubkey!("11111111111111111111111111111111");

#[program]
pub mod trustpay {
    use super::*;
    use crate::instructions::{
        crear_escrow::CrearEscrow,
        liberar_escrow::LiberarEscrow,
        registrar_negocio::RegistrarNegocio,
        verificar_negocio::VerificarNegocio,
    };

    pub fn registrar_negocio(ctx: Context<RegistrarNegocio>, business_id: String) -> Result<()> {
        crate::instructions::registrar_negocio::handler(ctx, business_id)
    }

    pub fn verificar_negocio(ctx: Context<VerificarNegocio>) -> Result<()> {
        crate::instructions::verificar_negocio::handler(ctx)
    }

    pub fn crear_escrow(
        ctx: Context<CrearEscrow>,
        transaction_id: String,
        amount: u64,
    ) -> Result<()> {
        crate::instructions::crear_escrow::handler(ctx, transaction_id, amount)
    }

    pub fn liberar_escrow(ctx: Context<LiberarEscrow>) -> Result<()> {
        crate::instructions::liberar_escrow::handler(ctx)
    }
}
