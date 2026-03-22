use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("6R5q6P5975rhYg2KXaGLPjysYBtMHGwqBajmYDJLwVih");

// Pubkey del keypair del backend TrustPay (authority).
// Reemplazar con la clave real antes de compilar.
const TRUSTPAY_AUTHORITY: Pubkey = pubkey!("11111111111111111111111111111111");

// ── Estados del Escrow ────────────────────────────────────────────────────────
const ESCROW_LOCKED: u8 = 1;
const ESCROW_RELEASED: u8 = 2;

#[program]
pub mod trustpay {
    use super::*;

    //////////////////////////// Instrucción: Registrar Negocio ////////////////////////////
    /*
    Crea la Business PDA de un merchant. El backend (authority) paga la cuenta
    y firma la transacción. La wallet del merchant solo se usa como semilla.

    Parámetros:
        * business_id -> UUID generado en PostgreSQL (36 chars)
    */
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
    /*
    Marca el negocio como verificado. Solo la authority de TrustPay puede llamar
    esta instrucción (validado con constraint en el contexto).
    */
    pub fn verificar_negocio(ctx: Context<VerificarNegocio>) -> Result<()> {
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
    /*
    Crea la Escrow PDA y bloquea los fondos. El buyer firma y paga la cuenta.
    Los lamports quedan retenidos en la PDA hasta que se liberen o reembolsen.

    Parámetros:
        * transaction_id -> UUID de la transacción en PostgreSQL (36 chars)
        * amount         -> Monto en lamports a bloquear
    */
    pub fn crear_escrow(
        ctx: Context<CrearEscrow>,
        transaction_id: String,
        amount: u64,
    ) -> Result<()> {
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

    //////////////////////////// Instrucción: Liberar Escrow ////////////////////////////
    /*
    El buyer confirma la recepción del producto y libera los fondos al seller.
    Solo el buyer puede llamar esta instrucción.
    */
    pub fn liberar_escrow(ctx: Context<LiberarEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_pda;

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            Errores::NoEresElBuyer
        );
        require!(escrow.status == ESCROW_LOCKED, Errores::EstadoInvalido);

        let amount = escrow.amount;

        // Transferir lamports de la PDA al seller
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
}

// ==============================
// ERRORES PERSONALIZADOS
// ==============================

#[error_code]
pub enum Errores {
    #[msg("Error: solo TrustPay puede verificar negocios")]
    NoAutorizado,

    #[msg("Error: este negocio ya fue verificado")]
    YaVerificado,

    #[msg("Error: el escrow no está en estado LOCKED")]
    EstadoInvalido,

    #[msg("Error: solo el comprador puede liberar el escrow")]
    NoEresElBuyer,
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

#[derive(Accounts)]
#[instruction(transaction_id: String)]
pub struct CrearEscrow<'info> {
    /// El comprador firma y paga la creación del escrow
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
