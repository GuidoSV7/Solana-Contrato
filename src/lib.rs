use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::{
    crear_escrow::*, liberar_escrow::*, registrar_negocio::*, verificar_negocio::*,
};

declare_id!("6R5q6P5975rhYg2KXaGLPjysYBtMHGwqBajmYDJLwVih");

// Pubkey del keypair del backend TrustPay (authority).
// Reemplazar con la clave real antes de compilar:
//   En Solana Playground: Settings > Wallet > tu pubkey
pub const TRUSTPAY_AUTHORITY: Pubkey = pubkey!("11111111111111111111111111111111");

#[program]
pub mod trustpay {
    use super::*;

    pub fn registrar_negocio(ctx: Context<RegistrarNegocio>, business_id: String) -> Result<()> {
        instructions::registrar_negocio::handler(ctx, business_id)
    }

    pub fn verificar_negocio(ctx: Context<VerificarNegocio>) -> Result<()> {
        instructions::verificar_negocio::handler(ctx)
    }

    pub fn crear_escrow(
        ctx: Context<CrearEscrow>,
        transaction_id: String,
        amount: u64,
    ) -> Result<()> {
        instructions::crear_escrow::handler(ctx, transaction_id, amount)
    }

    pub fn liberar_escrow(ctx: Context<LiberarEscrow>) -> Result<()> {
        instructions::liberar_escrow::handler(ctx)
    }
}
