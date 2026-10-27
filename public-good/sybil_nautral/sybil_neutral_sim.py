import numpy as np
rng=np.random.default_rng(3)
def L(s=""): print(s)
N=200_000; S=30e6*N/1e6; CAP=10_000; E=1.5
ask=rng.lognormal(np.log(60),1.5,N)          # honest demand at base price p0 (DK)
L(f"SYBIL-NOTR ONSATIS TASARIMI: {N:,} durust alici, arz {S/1e6:.1f}M DK, durust talep toplami {ask.sum()/1e6:.1f}M DK (taban fiyatta)")
L(" Karsilastirilan kurallar: (A) cuzdan basina tavan + suyla doldurma [mevcut], (B) sabit fiyat, paraya orantili pay, (C) fiyat-temizleyen satis: fiyat, talep=arz olana kadar yukselir (talep ~ (p0/p)^1.5), herkes ayni fiyattan alir.")
def alloc_A(asks,wal_owner=None):
    a=np.minimum(asks,CAP); lo,hi=0,CAP
    for _ in range(60):
        m=(lo+hi)/2
        if np.minimum(a,m).sum()>S: hi=m
        else: lo=m
    return np.minimum(a,lo), lo
def run(att_money_share, wallets_per_money_unit):
    # saldirgan: durust toplam paranin att_money_share'i kadar para; A'da cok cuzdana boler
    M_att=att_money_share*ask.sum()/(1-att_money_share)       # saldirganin talep butcesi (DK-esdegeri, taban fiyat)
    res={}
    # A: saldirgan W cuzdan acar, her biri seviye kadar ister (paras iyeterse). Seviye icin iteratif.
    W=int(min(5_000_000,wallets_per_money_unit))
    asks_all=np.concatenate([ask,np.full(W,min(CAP,M_att/max(W,1)))])
    a,lvl=alloc_A(asks_all)
    att=a[N:].sum(); hon=a[:N]
    res["A"]=(att/S, np.median(hon), hon[ask>=np.percentile(ask,99)].sum()/S, W)
    # B: sabit fiyat, paraya orantili (herkesin talebi para)
    tot=ask.sum()+M_att; fill=min(1.0,S/tot)
    res["B"]=(M_att*fill/S, np.median(ask*fill), (ask[ask>=np.percentile(ask,99)]*fill).sum()/S, 1)
    # C: fiyat temizleyen: talep(p)=Dtot*(p0/p)^E = S
    ratio=(tot/S)**(1/E); ratio=max(ratio,1.0); fillC=tot/ratio**E; frac=min(1,S/tot)
    q_att=M_att*ratio**-E; q_hon=ask*ratio**-E
    res["C"]=(q_att/S, np.median(q_hon), q_hon[ask>=np.percentile(ask,99)].sum()/S, ratio)
    return res
L(f"\n {'saldirgan para payi':>18} {'cuzdan':>9} | {'A: saldirgan payi':>17} {'medyan durust':>13} | {'B: saldirgan payi':>17} {'medyan':>7} | {'C: saldirgan payi':>17} {'medyan':>7} {'fiyat x':>7}")
for s in (0.0,0.01,0.05,0.20):
    for W in (1,1000,100_000):
        if s==0 and W>1: continue
        r=run(s,W)
        a,b,c=r["A"],r["B"],r["C"]
        L(f" {s:>18.0%} {W:>9,} | {a[0]:>17.1%} {a[1]:>13.1f} | {b[0]:>17.1%} {b[1]:>7.1f} | {c[0]:>17.1%} {c[1]:>7.1f} {c[3]:>7.2f}")
L("\n Okuma:")
L(" - A: saldirganin payi cuzdan sayisiyla buyur (parcalama isler); durust medyan alici dusur. Tavan sadece parcalamayan DURUST zenginleri cezalandirir.")
L(" - B ve C: saldirganin payi cuzdan sayisindan BAGIMSIZ (sadece para payi). Parcalamak kazandirmaz.")
L(" - C, B ile ayni dagilimi verir (sabit arz, paraya orantili); fark fiyatta: herkes temizleme fiyatini oder (fiyat x), hazine ~o kadar fazla toplar. Kucuk alicinin payi iki kuralda da ~%16 (A sybil yokken %62 - A durust kullanici icin daha cömert ama parcalamada coker).")
L(" - Taviz (C): ilk fiyat belirsiz, 'ne kadar odeyecegim' onceden bilinmez; talep esnekligi (1.5) VARSAYIM; gercek talep olculmedi.")
