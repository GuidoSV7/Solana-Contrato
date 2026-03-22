use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BusinessPda {
    pub owner: Pubkey, // Wallet del merchant (32 bytes)

    #[max_len(36)]
    pub business_id: String, // UUID de PostgreSQL — puente entre BD y blockchain

    pub is_verified: bool, // TrustPay confirmó que el negocio es legítimo

    pub bump: u8, // Identificador único de la PDA
}
