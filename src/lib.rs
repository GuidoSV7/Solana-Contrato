use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("6R5q6P5975rhYg2KXaGLPjysYBtMHGwqBajmYDJLwVih");

const TRUSTPAY_AUTHORITY: [u8; 32] = [
    191, 42, 247, 41, 178, 96, 177, 17, 50, 163, 79, 51, 237, 228, 158, 144, 208, 6, 65, 101, 145,
    167, 249, 89, 240, 42, 222, 13, 151, 134, 218, 195,
];

// ── Estados del Escrow ────────────────────────────────────────────────────────
// El estado CLOSED no existe como valor en la PDA porque cerrar_escrow
// destruye la cuenta on-chain vía `close = authority`.
const ESCROW_LOCKED: u8 = 1;
const ESCROW_RELEASED: u8 = 2;
const ESCROW_REFUNDED: u8 = 3;

#[program]
pub mod trustpay {
    use super::*;

    //////////////////////////// Instrucción: Registrar Negocio ////////////////////////////
    pub fn registrar_negocio(ctx: Context<RegistrarNegocio>, business_id: String) -> Result<()> {
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

    //////////////////////////// Instrucción: Verificar Negocio ////////////////////////////
    pub fn verificar_negocio(ctx: Context<VerificarNegocio>) -> Result<()> {
        require!(
            ctx.accounts.authority.key().to_bytes() == TRUSTPAY_AUTHORITY,
            Errores::NoAutorizado
        );

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

    //////////////////////////// Instrucción: Crear Escrow ////////////////////////////
    pub fn crear_escrow(
        ctx: Context<CrearEscrow>,
        transaction_id: String,
        amount: u64,
    ) -> Result<()> {
        // El bloque libera el borrow mutable antes del CPI,
        // que necesita acceso inmutable a escrow_pda.to_account_info().
        {
            let escrow = &mut ctx.accounts.escrow_pda;
            escrow.buyer = ctx.accounts.buyer.key();
            escrow.seller = ctx.accounts.seller.key();
            escrow.amount = amount;
            escrow.status = ESCROW_LOCKED;
            escrow.transaction_id = transaction_id.clone();
            escrow.bump = ctx.bumps.escrow_pda;
        }

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
            ctx.accounts.escrow_pda.buyer,
            ctx.accounts.escrow_pda.seller,
            amount
        );
        Ok(())
    }

    //////////////////////////// Instrucción: Liberar Escrow ////////////////////////////
    pub fn liberar_escrow(ctx: Context<LiberarEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_pda;

        // La validación del buyer ya la garantizan los seeds de la PDA:
        // si se pasa un buyer incorrecto, Anchor falla con "seeds constraint violated".
        require!(escrow.status == ESCROW_LOCKED, Errores::EstadoInvalido);

        let amount = escrow.amount;

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

    //////////////////////////// Instrucción: Reembolsar Escrow ////////////////////////////
    pub fn reembolsar_escrow(ctx: Context<ReembolsarEscrow>) -> Result<()> {
        require!(
            ctx.accounts.authority.key().to_bytes() == TRUSTPAY_AUTHORITY,
            Errores::NoAutorizadoReembolso
        );

        let escrow = &mut ctx.accounts.escrow_pda;

        require!(escrow.status == ESCROW_LOCKED, Errores::EstadoInvalido);

        let amount = escrow.amount;

        **escrow.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.buyer.try_borrow_mut_lamports()? += amount;

        escrow.status = ESCROW_REFUNDED;

        msg!(
            "Escrow reembolsado: {} | buyer: {} | amount: {} lamports",
            escrow.transaction_id,
            escrow.buyer,
            amount
        );
        Ok(())
    }

    //////////////////////////// Instrucción: Cerrar Escrow ////////////////////////////
    pub fn cerrar_escrow(ctx: Context<CerrarEscrow>) -> Result<()> {
        require!(
            ctx.accounts.authority.key().to_bytes() == TRUSTPAY_AUTHORITY,
            Errores::NoAutorizado
        );

        let escrow = &ctx.accounts.escrow_pda;

        require!(
            escrow.status == ESCROW_RELEASED || escrow.status == ESCROW_REFUNDED,
            Errores::NoSePuedeCerrar
        );

        msg!(
            "Escrow cerrado: {} | rent devuelto a TrustPay",
            escrow.transaction_id
        );
        Ok(())
    }
}

// ==============================
// ERRORES PERSONALIZADOS
// ==============================

#[error_code]
pub enum Errores {
    #[msg("Error: solo TrustPay puede realizar esta acción")]
    NoAutorizado,

    #[msg("Error: este negocio ya fue verificado")]
    YaVerificado,

    #[msg("Error: el escrow no está en estado LOCKED")]
    EstadoInvalido,

    #[msg("Error: solo TrustPay puede reembolsar el escrow")]
    NoAutorizadoReembolso,

    #[msg("Error: el escrow debe estar RELEASED o REFUNDED para cerrarse")]
    NoSePuedeCerrar,
}

// ==============================
// CUENTAS ON-CHAIN
// ==============================

#[account]
#[derive(InitSpace)]
pub struct BusinessPda {
    pub owner: Pubkey,

    #[max_len(36)]
    pub business_id: String,

    pub is_verified: bool,

    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct EscrowPda {
    pub buyer: Pubkey,

    pub seller: Pubkey,

    pub amount: u64,

    pub status: u8,

    #[max_len(36)]
    pub transaction_id: String,

    pub bump: u8,
}

// ==============================
// CONTEXTOS DE INSTRUCCIONES
// ==============================

#[derive(Accounts)]
#[instruction(business_id: String)]
pub struct RegistrarNegocio<'info> {
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

#[derive(Accounts)]
pub struct VerificarNegocio<'info> {
    #[account(mut)]
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

#[derive(Accounts)]
#[instruction(transaction_id: String)]
pub struct CrearEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: wallet del vendedor — se almacena en la PDA
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

#[derive(Accounts)]
pub struct LiberarEscrow<'info> {
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

#[derive(Accounts)]
pub struct ReembolsarEscrow<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: wallet del comprador — recibe el reembolso
    #[account(mut)]
    pub buyer: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"escrow", buyer.key().as_ref(), escrow_pda.transaction_id.as_bytes()],
        bump = escrow_pda.bump
    )]
    pub escrow_pda: Account<'info, EscrowPda>,
}

#[derive(Accounts)]
pub struct CerrarEscrow<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
        seeds = [b"escrow", escrow_pda.buyer.as_ref(), escrow_pda.transaction_id.as_bytes()],
        bump = escrow_pda.bump
    )]
    pub escrow_pda: Account<'info, EscrowPda>,
}