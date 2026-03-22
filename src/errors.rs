use anchor_lang::prelude::*;

#[error_code]
pub enum Errores {
    // ── Business PDA ──────────────────────────────────────────────────────────
    #[msg("Error: solo TrustPay puede verificar negocios")]
    NoAutorizado,

    #[msg("Error: este negocio ya fue verificado")]
    YaVerificado,

    // ── Escrow PDA ────────────────────────────────────────────────────────────
    #[msg("Error: el escrow no está en el estado correcto para esta operación")]
    EstadoInvalido,

    #[msg("Error: solo el comprador puede liberar el escrow")]
    NoEresElBuyer,
}
