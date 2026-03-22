use anchor_lang::prelude::*;

// ── Estados del Escrow ────────────────────────────────────────────────────────
pub const ESCROW_PENDING: u8 = 0;  // creado, esperando fondos
pub const ESCROW_LOCKED: u8 = 1;   // fondos bloqueados
pub const ESCROW_RELEASED: u8 = 2; // pago liberado al vendedor
pub const ESCROW_DISPUTED: u8 = 3; // disputa abierta
pub const ESCROW_REFUNDED: u8 = 4; // fondos devueltos al comprador

#[account]
#[derive(InitSpace)]
pub struct EscrowPda {
    pub buyer: Pubkey,  // Wallet del comprador

    pub seller: Pubkey, // Wallet del vendedor

    pub amount: u64,    // Monto en lamports bloqueado en la PDA

    pub status: u8,     // Estado actual del escrow (ver constantes arriba)

    #[max_len(36)]
    pub transaction_id: String, // UUID de la transacción en BD — puente entre BD y blockchain

    pub bump: u8,       // Identificador único de la PDA
}
