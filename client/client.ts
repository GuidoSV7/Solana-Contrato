// ─────────────────────────────────────────────────────────────────────────────
// LECTOR DE BLOCKCHAIN — TrustPay en Solana Playground
//
// Pega este código en el panel "Client" de Solana Playground y presiona "Run".
// Muestra todo lo que está guardado on-chain para tu wallet actual.
// ─────────────────────────────────────────────────────────────────────────────

// ── 1. Derivar la dirección del BusinessPda (mismo esquema que el contrato) ──
// UUID del negocio en la API (GET /businesses o respuesta de POST). Sin esto la PDA no coincide.
const BUSINESS_ID = "Pega-aqui-el-uuid-del-negocio";
const crypto = require("crypto");
const businessIdSeed = crypto
  .createHash("sha256")
  .update(BUSINESS_ID, "utf8")
  .digest();

const [businessPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("business"), pg.wallet.publicKey.toBuffer(), businessIdSeed],
  pg.program.programId
);

console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
console.log("🔍 Buscando BusinessPda en la blockchain...");
console.log("   Tu wallet  :", pg.wallet.publicKey.toBase58());
console.log("   businessId :", BUSINESS_ID);
console.log("   PDA address:", businessPda.toBase58());
console.log("   PDA bump   :", bump);
console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

// ── 2. Leer la cuenta del BusinessPda ────────────────────────────────────────
try {
  const negocio = await pg.program.account.businessPda.fetch(businessPda);

  console.log("✅ Negocio encontrado on-chain:");
  console.log("   owner       :", negocio.owner.toBase58());
  console.log("   business_id :", negocio.businessId);
  console.log("   is_verified :", negocio.isVerified);
  console.log("   bump        :", negocio.bump);
  console.log("");
  console.log("🔗 Ver en Solana Explorer:");
  console.log(
    `   https://explorer.solana.com/address/${businessPda.toBase58()}?cluster=devnet`
  );
} catch (e) {
  console.log("⚠️  No se encontró un BusinessPda para tu wallet.");
  console.log(
    "   Asegúrate de que el backend ya hizo POST /businesses con esta wallet:"
  );
  console.log("   walletAddress:", pg.wallet.publicKey.toBase58());
  console.log("   Error:", e.message);
}
