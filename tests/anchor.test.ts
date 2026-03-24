// No imports needed: web3, anchor, pg and more are globally available
// pg.wallet actúa como TrustPay authority (sus bytes están en TRUSTPAY_AUTHORITY del contrato)

// ─────────────────────────────────────────────────────────────────────────────
// BUSINESS PDA
// ─────────────────────────────────────────────────────────────────────────────

describe("Business PDA", () => {
  const businessId = "550e8400-e29b-41d4-a716-446655440000";

  // Wallet del merchant — no firma, solo se usa como semilla
  const merchant = web3.Keypair.generate();

  // Misma derivación que el contrato: seeds = ["business", owner, sha256(utf8(business_id))]
  const crypto = require("crypto");
  const businessIdSeed = crypto
    .createHash("sha256")
    .update(businessId, "utf8")
    .digest();
  const [businessPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("business"), merchant.publicKey.toBuffer(), businessIdSeed],
    pg.program.programId
  );

  it("registrar_negocio: crea la PDA con is_verified = false", async () => {
    const tx = await pg.program.methods
      .registrarNegocio(businessId)
      .accounts({
        authority: pg.wallet.publicKey,
        owner: merchant.publicKey,
        businessPda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("registrar_negocio tx:", tx);

    const cuenta = await pg.program.account.businessPda.fetch(businessPda);

    assert(cuenta.owner.equals(merchant.publicKey), "owner debe ser la wallet del merchant");
    assert.equal(cuenta.businessId, businessId, "business_id debe coincidir");
    assert.equal(cuenta.isVerified, false, "is_verified debe ser false al registrar");

    console.log("✅ Negocio registrado:", {
      businessId: cuenta.businessId,
      owner: cuenta.owner.toBase58(),
      isVerified: cuenta.isVerified,
    });
  });

  it("verificar_negocio: cambia is_verified a true", async () => {
    const tx = await pg.program.methods
      .verificarNegocio(businessId)
      .accounts({
        authority: pg.wallet.publicKey,
        owner: merchant.publicKey,
        businessPda,
      })
      .rpc();

    console.log("verificar_negocio tx:", tx);

    const cuenta = await pg.program.account.businessPda.fetch(businessPda);

    assert.equal(cuenta.isVerified, true, "is_verified debe ser true después de verificar");

    console.log("✅ Negocio verificado:", cuenta.businessId);
  });

  it("verificar_negocio: falla si el negocio ya está verificado", async () => {
    try {
      await pg.program.methods
        .verificarNegocio(businessId)
        .accounts({
          authority: pg.wallet.publicKey,
          owner: merchant.publicKey,
          businessPda,
        })
        .rpc();

      assert.fail("Debería haber lanzado YaVerificado");
    } catch (err) {
      const msg = err.message ?? "";
      assert(
        msg.includes("YaVerificado") || msg.includes("ya fue verificado"),
        `Error inesperado: ${msg}`
      );
      console.log("✅ Error esperado: YaVerificado");
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// ESCROW PDA — flujo feliz: crear → liberar → cerrar
// ─────────────────────────────────────────────────────────────────────────────

describe("Escrow PDA — flujo liberar", () => {
  const transactionId = "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11";
  const amount = new BN(100_000_000); // 0.1 SOL en lamports

  // En este test el buyer es pg.wallet (tiene SOL del airdrop)
  const seller = web3.Keypair.generate();

  const [escrowPda] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      pg.wallet.publicKey.toBuffer(),
      Buffer.from(transactionId),
    ],
    pg.program.programId
  );

  it("crear_escrow: bloquea los fondos en la PDA", async () => {
    const tx = await pg.program.methods
      .crearEscrow(transactionId, amount)
      .accounts({
        buyer: pg.wallet.publicKey,
        seller: seller.publicKey,
        escrowPda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("crear_escrow tx:", tx);

    const cuenta = await pg.program.account.escrowPda.fetch(escrowPda);

    assert(cuenta.buyer.equals(pg.wallet.publicKey), "buyer incorrecto");
    assert(cuenta.seller.equals(seller.publicKey), "seller incorrecto");
    assert(cuenta.amount.eq(amount), "amount incorrecto");
    assert.equal(cuenta.status, 1, "status debe ser 1 (LOCKED)");
    assert.equal(cuenta.transactionId, transactionId, "transaction_id incorrecto");

    const escrowBalance = await pg.connection.getBalance(escrowPda);
    assert(escrowBalance >= amount.toNumber(), "la PDA no tiene los fondos bloqueados");

    console.log("✅ Escrow creado | balance PDA:", escrowBalance, "lamports");
  });

  it("liberar_escrow: transfiere fondos al seller y marca status = 2 (RELEASED)", async () => {
    const sellerAntes = await pg.connection.getBalance(seller.publicKey);

    const tx = await pg.program.methods
      .liberarEscrow()
      .accounts({
        buyer: pg.wallet.publicKey,
        seller: seller.publicKey,
        escrowPda,
      })
      .rpc();

    console.log("liberar_escrow tx:", tx);

    const cuenta = await pg.program.account.escrowPda.fetch(escrowPda);
    assert.equal(cuenta.status, 2, "status debe ser 2 (RELEASED)");

    const sellerDespues = await pg.connection.getBalance(seller.publicKey);
    assert(sellerDespues > sellerAntes, "el seller no recibió fondos");

    console.log("✅ Escrow liberado | seller recibió:", sellerDespues - sellerAntes, "lamports");
  });

  it("cerrar_escrow: destruye la PDA y devuelve rent a TrustPay", async () => {
    const authorityAntes = await pg.connection.getBalance(pg.wallet.publicKey);

    const tx = await pg.program.methods
      .cerrarEscrow()
      .accounts({
        authority: pg.wallet.publicKey,
        escrowPda,
      })
      .rpc();

    console.log("cerrar_escrow tx:", tx);

    const cuentaInfo = await pg.connection.getAccountInfo(escrowPda);
    assert.equal(cuentaInfo, null, "la cuenta debe haber sido eliminada on-chain");

    const authorityDespues = await pg.connection.getBalance(pg.wallet.publicKey);
    console.log("✅ Escrow cerrado | rent recuperado:", authorityDespues - authorityAntes, "lamports");
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// ESCROW PDA — flujo reembolso: crear → reembolsar → cerrar
// ─────────────────────────────────────────────────────────────────────────────

describe("Escrow PDA — flujo reembolso", () => {
  const transactionId = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
  const amount = new BN(50_000_000); // 0.05 SOL

  const seller = web3.Keypair.generate();

  const [escrowPda] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      pg.wallet.publicKey.toBuffer(),
      Buffer.from(transactionId),
    ],
    pg.program.programId
  );

  it("crear_escrow: bloquea fondos para el escrow de reembolso", async () => {
    const tx = await pg.program.methods
      .crearEscrow(transactionId, amount)
      .accounts({
        buyer: pg.wallet.publicKey,
        seller: seller.publicKey,
        escrowPda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    const cuenta = await pg.program.account.escrowPda.fetch(escrowPda);
    assert.equal(cuenta.status, 1, "status debe ser 1 (LOCKED)");

    console.log("✅ Escrow creado para prueba de reembolso | tx:", tx);
  });

  it("reembolsar_escrow: devuelve fondos al buyer y marca status = 3 (REFUNDED)", async () => {
    const buyerAntes = await pg.connection.getBalance(pg.wallet.publicKey);

    const tx = await pg.program.methods
      .reembolsarEscrow()
      .accounts({
        authority: pg.wallet.publicKey,
        buyer: pg.wallet.publicKey,
        escrowPda,
      })
      .rpc();

    console.log("reembolsar_escrow tx:", tx);

    const cuenta = await pg.program.account.escrowPda.fetch(escrowPda);
    assert.equal(cuenta.status, 3, "status debe ser 3 (REFUNDED)");

    console.log("✅ Escrow reembolsado");
  });

  it("cerrar_escrow: elimina la PDA reembolsada", async () => {
    const tx = await pg.program.methods
      .cerrarEscrow()
      .accounts({
        authority: pg.wallet.publicKey,
        escrowPda,
      })
      .rpc();

    const cuentaInfo = await pg.connection.getAccountInfo(escrowPda);
    assert.equal(cuentaInfo, null, "la cuenta debe haber sido eliminada on-chain");

    console.log("✅ Escrow reembolsado cerrado | tx:", tx);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// VALIDACIONES DE ERROR
// ─────────────────────────────────────────────────────────────────────────────

describe("Validaciones de error", () => {
  it("cerrar_escrow: falla si el escrow está en estado LOCKED", async () => {
    const transactionId = "11111111-1111-1111-1111-111111111111";
    const seller = web3.Keypair.generate();

    const [escrowPda] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        pg.wallet.publicKey.toBuffer(),
        Buffer.from(transactionId),
      ],
      pg.program.programId
    );

    // Crear el escrow (queda en LOCKED)
    await pg.program.methods
      .crearEscrow(transactionId, new BN(10_000_000))
      .accounts({
        buyer: pg.wallet.publicKey,
        seller: seller.publicKey,
        escrowPda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    // Intentar cerrarlo sin liberar ni reembolsar → debe fallar
    try {
      await pg.program.methods
        .cerrarEscrow()
        .accounts({
          authority: pg.wallet.publicKey,
          escrowPda,
        })
        .rpc();

      assert.fail("Debería haber lanzado NoSePuedeCerrar");
    } catch (err) {
      const msg = err.message ?? "";
      assert(
        msg.includes("NoSePuedeCerrar") || msg.includes("RELEASED o REFUNDED"),
        `Error inesperado: ${msg}`
      );
      console.log("✅ Error esperado: NoSePuedeCerrar");
    }
  });
});
