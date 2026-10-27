import numpy as np, math
rng=np.random.default_rng(5)
def L(s=""): print(s)
STEP=0.02; SPREAD=0.005; LG=math.log(1+STEP)
L("KARSILASTIRMA: Gemini 'QEF' tasarimi (spesifikasyon metnindeki haliyle) vs DK merdiven tasarimi vs duz AMM (referans)")
L("Not: QEF'te T_B dis DEX'te (AMM) islem gorur; T_A/P_A sadece muhasebedir. Spesifikasyonda T_A'nin disarida bir degeri ya da T_B'yi satin alan bir mekanizma YOK.\n")
# ---------------- S1 exit / backing
L("S1) CIKIS: tokenlarin x'i tek seferde satilirsa sahipler, satis oncesi (isaretli) degerlerinin yuzde kacini alir?")
L("    AMM/QEF: havuz derinligi d = havuzdaki para / piyasa degeri, ucret %0.3. QEF as-written = AMM (destek 0). DK: merdiven yarisi dolu (63/127 bin), saticilar egriden asagi iner, %0.5 spread.")
phi_bins=63
def ladder_recovery(x):
    top=phi_bins; lo=top*(1-x)
    proceeds=((1+STEP)**top-(1+STEP)**lo)/LG*(1-SPREAD)
    marked=(1+STEP)**top*(top-lo)
    return proceeds/marked
def amm_recovery(d,x,fee=0.003):
    Y=d  # token reserve in units of supply S (price=1 normalised, R=d*M, Y=R/p=d*S)
    R=d
    s=x*(1-fee)
    proceeds=R*(1-Y/(Y+s))
    return proceeds/x
L(f" {'satilan x':>9} | {'AMM/QEF d=5%':>13} {'d=20%':>7} {'d=50%':>7} | {'DK merdiven':>11}")
for x in (0.05,0.10,0.30,0.60):
    L(f" {x:>9.0%} | {amm_recovery(.05,x):>13.1%} {amm_recovery(.20,x):>7.1%} {amm_recovery(.50,x):>7.1%} | {ladder_recovery(x):>11.1%}")
resv=((1+STEP)**phi_bins-1)/LG; mcap=(1+STEP)**phi_bins*phi_bins
L(f" DK merdiven rezervi / piyasa degeri = {resv/mcap:.0%} (kendi derinligi; esdeger AMM d≈%57). Bu rezerv herkesin cikis egrisini kapsar (kurala gore).")
L(f" DK cikis HIZI: gunluk limit arzin %2'si, ladder'in yarisi dolu -> sahiplerin %30'u (arzin %15'i) ~7.5 gunde cikar (AMM: tek blokta ama yukaridaki fiyatla).")
# ---------------- S2 sandwich
L("\nS2) SANDVIC (MEV): saldirgan, kurbanin alisinin onune/arkasina girer. Kurban fiyat toleransi %1. Kar, kurban islem degerinin yuzdesi (negatif = saldiri zararli):")
def amm_sandwich(d,vfrac,tol=0.01,fee=0.003):
    R=d*1.0; Y=d*1.0; v=vfrac   # quote reserve R, token Y (price 1), victim buys v quote
    out0=Y-R*Y/(R+v*(1-fee)); best=0.0; bestloss=0.0
    for a in np.geomspace(v*0.01,v*200,400):
        R1=R+a*(1-fee); Y1=R*Y/R1; got_a=Y-Y1
        R2=R1+v*(1-fee); Y2=R1*Y1/R2; vout=Y1-Y2
        if vout<out0*(1-tol): continue
        R3=R2; Y3=Y2+got_a*(1-fee); R4=R2*Y2/Y3; a_back=R2-R4
        prof=a_back-a
        if prof>best: best=prof; bestloss=1-vout/out0
    return best/v, bestloss
def ladder_sandwich(vfrac_of_bin,tol=0.01):
    # bin = 1.0 token-unit; victim buys v bins; attacker a<=min(5 bins, tolerance-limited)
    C=lambda s:(1+STEP)**s/LG
    S=40.0; v=vfrac_of_bin; amax=min(5.0,math.log(1+tol)/LG)
    best=0.0; loss=0.0
    for a in np.linspace(0.001,amax,300):
        cost_a=C(S+a)-C(S)
        sell=(C(S+v+a)-C(S+v))*(1-SPREAD)
        prof=sell-cost_a
        if prof>best: best=prof; loss=(C(S+a+v)-C(S+a))-(C(S+v)-C(S))
    vv=C(S+v)-C(S)
    return best/vv, loss/vv
L(f" {'kurban/piyasa degeri':>20} | {'AMM d=20% kar':>13} {'kurban kaybi':>12} | {'DK kurban/bin':>13} {'DK kar':>8} {'kurban kaybi':>12}")
M=mcap
for vf in (1e-6,1e-5,1e-4,1e-3,1e-2):
    a,al=amm_sandwich(0.20,vf)
    vb=vf*M/((1+STEP)**phi_bins) if False else vf*phi_bins*(1+STEP)**phi_bins/ (1+STEP)**phi_bins*0+ vf*mcap/ ((1+STEP)**40)  # victim size in bins (token units)
    l,ll=ladder_sandwich(vb)
    L(f" {vf:>20.0e} | {a:>13.3%} {al:>12.3%} | {vb:>13.4f} {l:>8.3%} {ll:>12.3%}")
L(" Okuma: iki sistemde de kucuk islemler sandvicle karsiz (ucret/spread asilamiyor). Esik: AMM (derinlik %20) ~piyasa degerinin %0.1'i; DK merdiveni ~1 bin (~%1-2). Yani DK esigi ~10 kat yuksek, ama buyuk (balina) alislar DK'de de savunmasiz: cozum round modu (toplu, tek fiyat).")
# ---------------- S3 circuit breaker + fuzz
L("\nS3) QEF 'DEVRE KESICI': kaos = (hacim x prevrandao / 10^4) mod sigma; esik tau'yi asarsa 'Gozlem Modu'. mod sigma yuzunden her islemin tetikleme olasiligi hacimden BAGIMSIZ ~ (sigma-tau)/sigma.")
tr=[]
for p in (0.01,0.05,0.10):
    L(f"  tetik olasiligi {p:>4.0%}/islem:")
    for tx in (1_000,10_000,300_000):
        trips_per_day=tx*p; cool=10    # dk, varsayim
        frozen=1-math.exp(-trips_per_day*cool/1440) if trips_per_day>0 else 0
        L(f"    gunluk {tx:>7,} islem -> gunde {trips_per_day:>9,.0f} tetik; {cool} dk bekleme varsayimiyla zamanin ~{frozen:6.1%}'i dondurulmus")
    L(f"    saldirgan maliyeti: tetiklemek icin ~{1/p:.0f} kucuk islem x %0.3 ucret x 1 birim = ~{(1/p)*0.003:.2f} birim")
L(" Okuma: esik normal kullanimda da surekli asilir; sistem ya surekli donuk ya da esik kapatilirsa koruma sifir. DK'de karsilik: limit varlik payina gore paylasildiginda saldirgan para payi kadar engeller (unique_sonuc A2).")
L(" Fuzz (delta): yalniz P_A'ya eklenir; P_A piyasa fiyatini belirlemez (fiyat DEX'te). Dolayisiyla arbitraj botlarini etkilemez; ayrica girdileri (hacim, prevrandao) islemden once bilinir -> bot hesaplayabilir.")
# ---------------- scoreboard
L("\nOZET TABLO (simule edilen + spesifikasyonun kendisi)")
rows=[("Gercek teminat / cikis garantisi","YOK (P_A tanim geregi; T_B'yi destekleyen odeme yok)","Var (rezerv ≈ %57 piyasa degeri, egriden odeme)"),
("%30 satista geri alinan deger","%14-62 (derinlige bagli)","%83"),
("Cikis hizi","Aninda ama dusuk fiyat","Yavas (limit: ~7.5 gun) "),
("Cikis engelleme direnci","Zayif: devre kesici ~her islemde tetiklenir","Orta: limit varlik payina gore (para payi kadar engel)"),
("Sandvic","Karsiz kucukte; esik ~%0.1 piyasa degeri","Karsiz kucukte; esik ~%1-2 (10x yuksek); balina alislari acik"),
("Rastgelelik/kalkan","Ongorulebilir, etkisiz","Yok iddiasi yok; blur ayri katman"),
("Platform","Ethereum/Solidity","Solana (mevcut proje)"),
("Zayif noktalar","Dairesel muhasebe, tek hata noktasi","Gec satanlar kaybeder (bonding curve), yavas cikis, sybil, denetim yok")]
for r in rows: L(f" - {r[0]}: QEF -> {r[1]} | DK -> {r[2]}")
L("\nMODELLENMEDI: gercek fiyat/talep, dis likidite, MEV rekabeti, QEF'in 'duzeltilmis' (gercek rezervli) bir surumu, DK'nin talep sokunda surdurulebilirligi.")
