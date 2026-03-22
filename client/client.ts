// 1. CREAR PAPELERIA (C)
const [papeleriaPDA] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("papeleria"), pg.wallet.publicKey.toBuffer()],
  pg.program.programId
);

await pg.program.methods
  .crearPapeleria("Mi Papeleria Central")
  .accounts({
    owner: pg.wallet.publicKey,
    papeleria: papeleriaPDA,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
console.log("✅ Papeleria creada!");

// 2. AGREGAR PRODUCTO (C)
await pg.program.methods
  .agregarProducto("Lapiz HB", "Escritura", new anchor.BN(5000000), 100)
  .accounts({
    owner: pg.wallet.publicKey,
    papeleria: papeleriaPDA,
  })
  .rpc();
console.log("✅ Producto agregado!");

// 3. VER INVENTARIO (R)
const cuenta = await pg.program.account.papeleria.fetch(papeleriaPDA);
console.log("📦 Inventario:", cuenta.productos);

// 4. ACTUALIZAR STOCK (U)
await pg.program.methods
  .actualizarStock("Lapiz HB", 80)
  .accounts({
    owner: pg.wallet.publicKey,
    papeleria: papeleriaPDA,
  })
  .rpc();
console.log("✅ Stock actualizado!");

// 5. ELIMINAR PRODUCTO (D)
await pg.program.methods
  .eliminarProducto("Lapiz HB")
  .accounts({
    owner: pg.wallet.publicKey,
    papeleria: papeleriaPDA,
  })
  .rpc();
console.log("✅ Producto eliminado!");
