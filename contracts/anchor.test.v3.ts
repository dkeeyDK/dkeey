// Playground: tests/anchor.test.ts (v3 akisi). HENUZ CALISTIRILMADI: ilk derlemeden sonra calistirip hatalari birlikte duzeltecegiz.
// Gerekir: presale_program_v3.rs derlenmis ve deploy edilmis; cuzdanda ~2-3 devnet SOL.
import * as spl from "@solana/spl-token";
const P = pg.program.programId;
const SP = web3.SystemProgram.programId;
const TP = spl.TOKEN_PROGRAM_ID;
const WEEK = 15;
const le = (n: number | BN, b = 8) => new BN(n).toArrayLike(Buffer, "le", b);
const pda = (seeds: (Buffer | Uint8Array)[]) => web3.PublicKey.findProgramAddressSync(seeds, P)[0];
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
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

describe("dkeey v3: token, adil on satis, havuz, komite", function () {
  this.timeout(1200000);
  const id = Date.now();
  const sale = pda([Buffer.from("sale"), le(id)]);
  const demand = pda([Buffer.from("demand"), sale.toBuffer()]);
  const cfg = pda([Buffer.from("cfg"), sale.toBuffer()]);
  const tok = pda([Buffer.from("tok"), sale.toBuffer()]);
  const dkauth = pda([Buffer.from("dkauth"), sale.toBuffer()]);
  const mint = pda([Buffer.from("dkmint"), sale.toBuffer()]);
  const vmaster = pda([Buffer.from("vmaster"), sale.toBuffer()]);
  const vault = (k: number) => pda([Buffer.from("vault"), sale.toBuffer(), Buffer.from([k])]);
  const contrib = (u: web3.PublicKey) => pda([Buffer.from("c"), sale.toBuffer(), u.toBuffer()]);
  const reserve = pda([Buffer.from("reserve"), sale.toBuffer()]);
  const liq = pda([Buffer.from("liq"), sale.toBuffer()]);
  const ladder = pda([Buffer.from("ladder"), cfg.toBuffer()]);
  const gov = pda([Buffer.from("gov"), cfg.toBuffer()]);
  const lpv = pda([Buffer.from("lpv"), cfg.toBuffer()]);
  const lfee = pda([Buffer.from("lfee"), cfg.toBuffer()]);
  const whale = web3.Keypair.generate();
  const smalls = Array.from({ length: 4 }, () => web3.Keypair.generate());
  const all = [whale, ...smalls];
  const L = 1_000_000_000;
  const prices = [new BN(1_000_000), new BN(1_250_000), new BN(1_562_500)]; // lamport / DK
  let start = 0;

  it("kurulum: satis, token (1B sabit arz), kasalar", async () => {
    await fund(all, 0.2 * L);
    start = Math.floor(Date.now() / 1000) + 3;
    // oda/hafta = cap/3 = 3000 DK karsiligi; 5 cuzdan toplam 3x talep -> adil dagitim devreye girer
    const cap = new BN(3 * 3000 * 1_000_000), min = new BN(1_000_000_000);
    await pg.program.methods.initialize(new BN(id), new BN(start), new BN(WEEK), cap, min, prices as any)
      .accounts({ sale, demand, payer: pg.wallet.publicKey, systemProgram: SP }).rpc();
    await pg.program.methods.initToken()
      .accounts({ sale, tok, dkauth, mint, master: vmaster, authority: pg.wallet.publicKey, tokenProgram: TP, systemProgram: SP, rent: web3.SYSVAR_RENT_PUBKEY }).rpc();
    for (const k of [1, 2, 3, 4, 5]) {
      await pg.program.methods.initVault(k)
        .accounts({ sale, tok, dkauth, mint, vault: vault(k), authority: pg.wallet.publicKey, tokenProgram: TP, systemProgram: SP, rent: web3.SYSVAR_RENT_PUBKEY }).rpc();
    }
    const fundAcc = (dest: web3.PublicKey) => ({ tok, dkauth, master: vmaster, dest, authority: pg.wallet.publicKey, tokenProgram: TP });
    await pg.program.methods.fundPresale(new BN(30_000_000)).accounts(fundAcc(vault(1))).rpc();
    await pg.program.methods.fundLadder(new BN(100_000_000)).accounts(fundAcc(vault(2))).rpc();
    await pg.program.methods.fundReward(new BN(250_000_000)).accounts(fundAcc(vault(3))).rpc();
    const m = await spl.getMint(pg.connection, mint);
    assert(m.supply === 1_000_000_000n && m.mintAuthority === null, "arz sabit ve mint yetkisi kapali olmali");
    await fails(pg.program.methods.fundPresale(new BN(1)).accounts(fundAcc(vault(1))).rpc(), "on satis kasasi 30M ustu");
    console.log("  OK: 1B DK basildi, mint yetkisi kapatildi");
  });

  it("on satis: cuzdan basina 10K DK sinir + esit dagitim", async () => {
    const c = (u: web3.Keypair, amt: BN) => pg.program.methods.contribute(amt)
      .accounts({ sale, demand, contrib: contrib(u.publicKey), user: u.publicKey, systemProgram: SP }).signers([u]).rpc();
    while (Math.floor(Date.now() / 1000) < start) await sleep(500);
    await c(whale, new BN(10_000).mul(prices[0]));              // balina 10K DK ister
    for (const s of smalls) await c(s, new BN(1_000).mul(prices[0])); // kucukler 1K DK ister
    const cw = await pg.program.account.contrib.fetch(contrib(whale.publicKey));
    assert(cw.req[0].eq(new BN(10_000).mul(prices[0])), "balina istegi kaydedilmeli");
    await fails(c(whale, new BN(1).mul(prices[0])), "10K DK uzeri istek");
    // hafta 0 kapanir, hafta 1 ve 2 bos gecer
    for (let w = 0; w < 3; w++) {
      while (Math.floor(Date.now() / 1000) < start + (w + 1) * WEEK + 1) await sleep(500);
      await pg.program.methods.settleWeek(w).accounts({ sale, demand, caller: pg.wallet.publicKey }).rpc();
    }
    const s = await pg.program.account.sale.fetch(sale);
    const lvl = s.level[0];
    assert(lvl.lt(new BN(10_000).mul(prices[0])), "seviye balina isteginin altinda olmali");
    // beklenen: oda 3000 DK; 4 kucuk x 1000 = 4000 -> hepsi ~600 DK; balina da ayni seviyede
    console.log("  seviye (lamport):", lvl.toString(), " dagitilan:", s.raised.toString());
    await pg.program.methods.finalize().accounts({ sale }).rpc();
  });

  it("iade + token talebi (gercek DK)", async () => {
    const s = await pg.program.account.sale.fetch(sale);
    assert(s.success === (s.raised.gte(s.min)), "basari esige gore");
    const u = whale;
    const ata = await spl.getOrCreateAssociatedTokenAccount(pg.connection, u, mint, u.publicKey);
    const before = await pg.connection.getBalance(u.publicKey);
    await pg.program.methods.refund().accounts({ sale, contrib: contrib(u.publicKey), user: u.publicKey }).signers([u]).rpc();
    const after = await pg.connection.getBalance(u.publicKey);
    console.log("  iade (lamport):", after - before);
    await fails(pg.program.methods.refund().accounts({ sale, contrib: contrib(u.publicKey), user: u.publicKey }).signers([u]).rpc(), "ikinci iade");
    if (s.success) {
      await sleep(WEEK * 1000);
      await pg.program.methods.claim().accounts({ sale, contrib: contrib(u.publicKey), tok, dkauth, vault: vault(1), userDk: ata.address, user: u.publicKey, tokenProgram: TP }).signers([u]).rpc();
      const bal = await spl.getAccount(pg.connection, ata.address);
      assert(bal.amount > 0n, "gercek DK gelmeli");
      console.log("  DK bakiyesi:", bal.amount.toString());
    } else console.log("  (esik gecilmedi: tam iade; claim yok)");
  });

  // Havuz ve komite testleri (cfg, ladder, tur, geri satis, oy) derleme gecince buraya eklenecek:
  //  - init_cfg/init_ladder (daily_cap kucuk), set_step, buy_ladder, sell_ladder, gunluk tavan reddi
  //  - open_round/place_order x3/close_round/settle_order: orantili dolum, toplam dolum <= kapasite
  //  - init_gov/enable_gov/lock_votes/propose/vote/execute: tavan ve katilim, update_cfg'nin kapanmasi
});
