// Playground: tests/anchor.test.ts. Çok kullanıcılı devnet testleri (hafta=15 sn, küçük tutarlar).
// Her senaryo kendi satış kimliğiyle çalışır, yeniden çalıştırılabilir.
const L = 1_000_000_000;
const WEEK = 15;
const SP = web3.SystemProgram.programId;
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const idBuf = (id: number) => new BN(id).toArrayLike(Buffer, "le", 8);
const salePda = (id: number) =>
  web3.PublicKey.findProgramAddressSync([Buffer.from("sale"), idBuf(id)], pg.program.programId)[0];
const contribPda = (sale: web3.PublicKey, u: web3.PublicKey) =>
  web3.PublicKey.findProgramAddressSync([Buffer.from("c"), sale.toBuffer(), u.toBuffer()], pg.program.programId)[0];
const bn = (x: number) => new BN(Math.round(x));
const fails = async (p: Promise<any>, name: string) => {
  let f = false;
  try { await p; } catch (e) { f = true; }
  assert(f, "BEKLENEN HATA OLMADI: " + name);
  console.log("  OK reddedildi:", name);
};
const fund = async (kps: web3.Keypair[], each: number) => {
  const tx = new web3.Transaction();
  for (const k of kps) tx.add(web3.SystemProgram.transfer({ fromPubkey: pg.wallet.publicKey, toPubkey: k.publicKey, lamports: each }));
  await web3.sendAndConfirmTransaction(pg.connection, tx, [pg.wallet.keypair]);
};
const init = async (id: number, start: number, cap: number, min: number) =>
  pg.program.methods.initialize(new BN(id), new BN(start), new BN(WEEK), bn(cap), bn(min), [new BN(1000), new BN(800), new BN(640)])
    .accounts({ sale: salePda(id), payer: pg.wallet.publicKey, systemProgram: SP }).rpc();
const put = (id: number, kp: web3.Keypair, amt: number) =>
  pg.program.methods.contribute(bn(amt))
    .accounts({ sale: salePda(id), contrib: contribPda(salePda(id), kp.publicKey), user: kp.publicKey, systemProgram: SP })
    .signers([kp]).rpc();
const fin = (id: number) => pg.program.methods.finalize().accounts({ sale: salePda(id) }).rpc();
const refund = (id: number, kp: web3.Keypair, ownerOfContrib?: web3.PublicKey) =>
  pg.program.methods.refund()
    .accounts({ sale: salePda(id), contrib: contribPda(salePda(id), ownerOfContrib ?? kp.publicKey), user: kp.publicKey })
    .signers([kp]).rpc();
const reservePda = (sale: web3.PublicKey) =>
  web3.PublicKey.findProgramAddressSync([Buffer.from("reserve"), sale.toBuffer()], pg.program.programId)[0];
const liqPda = (sale: web3.PublicKey) =>
  web3.PublicKey.findProgramAddressSync([Buffer.from("liq"), sale.toBuffer()], pg.program.programId)[0];
const release = (id: number) =>
  pg.program.methods.release()
    .accounts({ sale: salePda(id), reserve: reservePda(salePda(id)), liquidity: liqPda(salePda(id)), caller: pg.wallet.publicKey }).rpc();
const claim = (id: number, kp: web3.Keypair) =>
  pg.program.methods.claim()
    .accounts({ sale: salePda(id), contrib: contribPda(salePda(id), kp.publicKey), user: kp.publicKey })
    .signers([kp]).rpc();
const users = (n: number) => Array.from({ length: n }, () => web3.Keypair.generate());
const waitUntil = async (t: number) => { const d = t - Math.floor(Date.now() / 1000); if (d > 0) await sleep(d * 1000); };
const raisedOf = async (id: number) => (await pg.program.account.sale.fetch(salePda(id))).raised.toNumber();

describe("dkeey_presale çok kullanıcılı", function () {
  this.timeout(900000);

  it("S2-S4: 10 kullanıcı, eşzamanlı, haftalık tavan, başarılı kapanış, tavan sonrası kapama", async () => {
    const id = Date.now();
    const cap = 0.06 * L, min = 0.03 * L, room = cap / 3;
    const U = users(10);
    await fund(U, 0.03 * L);
    const start = Math.floor(Date.now() / 1000) + 2;
    await init(id, start, cap, min);
    await waitUntil(start + 1);
    for (let w = 0; w < 3; w++) {
      // her hafta 10 kullanıcı eşzamanlı 0.01 SOL yollar; yer sadece 0.02 SOL
      const res = await Promise.allSettled(U.map((u) => put(id, u, 0.01 * L)));
      const ok = res.filter((r) => r.status === "fulfilled").length;
      const raised = await raisedOf(id);
      console.log(`  hafta ${w}: başarılı ${ok}, raised ${raised}`);
      assert(raised === (w + 1) * room, "hafta " + w + " birikim " + raised + " olmalı " + (w + 1) * room);
      if (w < 2) await waitUntil(start + (w + 1) * WEEK + 1);
    }
    // tavana ulaşıldı: yeni katkı reddedilir, erken finalize başarılı olur
    await fails(put(id, U[0], 0.01 * L), "tavan sonrası katkı");
    await fin(id);
    const s = await pg.program.account.sale.fetch(salePda(id));
    assert(s.finalized && s.success, "başarılı kapanmalı");
    // iade başarılı satışta yapılamaz
    for (const u of U.slice(0, 3)) await fails(refund(id, u), "başarılı satışta iade");
    // değişmez: hesapların toplamı = raised, hesap bakiyesi = kira + raised
    let sum = 0;
    for (const u of U) { try { sum += (await pg.program.account.contrib.fetch(contribPda(salePda(id), u.publicKey))).amount.toNumber(); } catch (e) {} }
    assert(sum === s.raised.toNumber(), "katkı toplamı raised ile eşit olmalı: " + sum);
    const lam = await pg.connection.getBalance(salePda(id));
    const rent = await pg.connection.getMinimumBalanceForRentExemption(92);
    assert(lam === rent + s.raised.toNumber(), "emanet bakiyesi kira+raised olmalı: " + lam);
    console.log("  OK değişmezler: toplam=raised, bakiye=kira+raised");
    // release: %75 rezerv, %25 likidite, bir kez, kasada yalnızca kira kalır
    const total = s.raised.toNumber();
    await release(id);
    const resBal = await pg.connection.getBalance(reservePda(salePda(id)));
    const liqBal = await pg.connection.getBalance(liqPda(salePda(id)));
    assert(resBal === Math.floor(total * 75 / 100), "rezerv %75 olmalı: " + resBal);
    assert(resBal + liqBal === total, "rezerv+likidite = raised");
    assert((await pg.connection.getBalance(salePda(id))) === rent, "release sonrası emanette yalnızca kira kalmalı");
    await fails(release(id), "çift release");
    // claim: doğrusal açılım (8 hafta = 120 sn). Yalnızca hakkı olan hesap, birikim zamanla artar, hak aşılmaz.
    let holder: web3.Keypair | undefined;
    for (const u of U) { try { await pg.program.account.contrib.fetch(contribPda(salePda(id), u.publicKey)); holder = u; break; } catch (e) {} }
    assert(holder, "katkısı olan bir kullanıcı bulunmalı");
    await claim(id, holder!);
    const c1 = (await pg.program.account.contrib.fetch(contribPda(salePda(id), holder!.publicKey)));
    assert(c1.claimed.toNumber() > 0 && c1.claimed.toNumber() < c1.tokens.toNumber(), "ilk claim hakkın küçük bir kısmı olmalı: " + c1.claimed.toNumber());
    await sleep(4000);
    await claim(id, holder!);
    const c2 = (await pg.program.account.contrib.fetch(contribPda(salePda(id), holder!.publicKey)));
    assert(c2.claimed.toNumber() > c1.claimed.toNumber() && c2.claimed.toNumber() <= c2.tokens.toNumber(), "claim zamanla artmalı ve hakkı aşmamalı");
    const stranger = web3.Keypair.generate();
    await fund([stranger], 0.01 * L);
    await fails(claim(id, stranger), "katkısız hesabın claim'i");
    console.log("  OK release ve claim");
  });

  it("S5: eşik altı, çok kullanıcılı tam iade, çift iade, yabancı iade", async () => {
    const id = Date.now() + 1;
    const cap = 0.06 * L, min = 0.06 * L;
    const U = users(8);
    await fund(U, 0.03 * L);
    const start = Math.floor(Date.now() / 1000) + 2;
    await init(id, start, cap, min);
    await waitUntil(start + 1);
    await Promise.allSettled(U.map((u) => put(id, u, 0.005 * L)));
    const raised = await raisedOf(id);
    assert(raised === 0.02 * L, "hafta0 raised 0.02 olmalı: " + raised);
    const mine: Record<string, number> = {};
    for (const u of U) { try { mine[u.publicKey.toBase58()] = (await pg.program.account.contrib.fetch(contribPda(salePda(id), u.publicKey))).amount.toNumber(); } catch (e) {} }
    // iade, finalize öncesi reddedilir
    await fails(refund(id, U[0]), "finalize öncesi iade");
    await waitUntil(start + 3 * WEEK + 2);
    await fin(id);
    await fails(fin(id), "çift finalize");
    await fails(release(id), "başarısız satışta release");
    await fails(claim(id, U[0]), "başarısız satışta claim");
    // yabancı hesap: U[7] katkı yapmış olabilir; yapmamış biri başkasının hesabından çekmeye çalışır
    const holder = U.find((u) => (mine[u.publicKey.toBase58()] ?? 0) > 0)!;
    const thief = web3.Keypair.generate();
    await fund([thief], 0.01 * L);
    await fails(refund(id, thief, holder.publicKey), "başkasının iadesini çekme");
    // gerçek iadeler
    let total = 0;
    for (const u of U) {
      const amt = mine[u.publicKey.toBase58()] ?? 0;
      if (amt === 0) continue;
      const b = await pg.connection.getBalance(u.publicKey);
      await refund(id, u);
      const a = await pg.connection.getBalance(u.publicKey);
      assert(a - b === amt - 0 || Math.abs(a - b - amt) <= 10000, "iade tutarı katkıya eşit olmalı: " + (a - b) + " vs " + amt);
      total += amt;
    }
    assert(total === raised, "toplam iade = raised");
    const lam = await pg.connection.getBalance(salePda(id));
    const rent = await pg.connection.getMinimumBalanceForRentExemption(92);
    assert(lam === rent, "iadeden sonra emanette sadece kira kalmalı: " + lam + " vs " + rent);
    await fails(refund(id, holder), "çift iade");
    console.log("  OK: tüm katkılar tam iade edildi, emanet sıfırlandı");
  });

  it("S6: kenar durumları", async () => {
    const id = Date.now() + 2;
    const U = users(2);
    await fund(U, 0.02 * L);
    const start = Math.floor(Date.now() / 1000) + 2;
    await fails(init(id, start, 0.03 * L, 0.06 * L), "min > cap");
    await init(id, start, 0.06 * L, 0.03 * L);
    await fails(put(id, U[0], 0.01 * L), "başlamadan katkı");
    await waitUntil(start + 1);
    await fails(put(id, U[0], 0), "sıfır katkı");
    await put(id, U[0], 0.005 * L);
    await fails(init(id, start, 0.06 * L, 0.03 * L), "aynı kimlikle ikinci kurulum");
    console.log("  OK kenar durumları");
  });
});
