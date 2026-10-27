import numpy as np, sys
rng=np.random.default_rng(7)
def L(s=""): print(s)
# ---------- A) aging weight ----------
L("A) PAYLASIM AGIRLIGI: odul / cikis limiti havuzu nasil bolunur? (100,000 durust cuzdan, yas 0-180 gun, sapma lognormal)")
H=100_000
hold=rng.lognormal(np.log(60),1.5,H); age=rng.uniform(0,180,H)
A=90.0; floor=0.05
f=lambda a: np.maximum(floor,np.minimum(a,A)/A)
def shares(att_hold,W,att_age):
    # attacker splits att_hold over W new wallets of age att_age
    ah=np.full(W,att_hold/W)
    out={}
    out["esit(cuzdan basina)"]=W/(H+W)
    out["varlik"]=att_hold/(hold.sum()+att_hold)
    wh=(hold*f(age)).sum(); wa=att_hold*f(att_age)
    out["varlik x yas"]=wa/(wh+wa)
    return out
tot=hold.sum()
for s in (0.01,0.05):
    B=s*tot
    L(f" Saldirgan parasi = durust toplamin {s:.0%}'i")
    for W in (1,100,10_000):
        for a in (0,30,90):
            r=shares(B,W,a)
            L(f"   {W:>6} cuzdan, yas {a:>2} gun -> havuz payi: "+" | ".join(f"{k} {v:6.1%}" for k,v in r.items()))
# inequality
def top1(w): w=np.sort(w)[::-1]; return w[:H//100].sum()/w.sum()
L(f" Durust ust %1'in havuz payi: esit {1/100:.1%} | varlik {top1(hold):.1%} | varlik x yas {top1(hold*f(age)):.1%}")
w=hold*f(age); med=np.median(w/w.sum()); L(f" Medyan kullanicinin payi (esit=1.00 alinirsa): varlik {np.median(hold/hold.sum())*H:.2f} | varlik x yas {med*H:.2f}")
L(" Okuma: parcalamak (1 -> 10,000 cuzdan) varlik veya varlik x yas semasinda HIC avantaj vermez; esit semada payi ~90x yapar. Yas, 'yeni parayla hemen saldirmayi' yasla/sermaye bekletmeyle pahalilastirir.")
# exit denial under weighted cap
L("\nA2) CIKIS ENGELLEME: limit agirliga gore paylasilir; saldirgan kendi payini satip geri alarak o payi bosa harcar (cap 2.0M DK/gun, spread %0.5)")
cap=2_000_000
for s in (0.01,0.05,0.2):
    B=s*tot*10  # ayni sermaye orani, 1M kullanici olceginde yaklasik
    for a in (0,90):
        sh=shares(s*tot,1,a)["varlik x yas"]
        L(f"   saldirgan para payi {s:>4.0%}, yas {a:>2}: baskalarindan alinan limit payi {sh:5.1%}  maliyet/gun ~{0.005*cap*sh:,.0f} birim")
L(" Karsilastirma: tek global ilk-gelen limit: baskalarindan alinan pay %100, maliyet ~22,080 birim/gun (milyon-sim S4).")
# ---------- B) blur at small N ----------
L("\nB) BULANIKLIK KUCUK KULLANICI SAYISINDA (tur basina en az 20 islem; 0.3 islem/kullanici/gun)")
need=20; size=120; spread=0.005; daycap=143_000/7
L(f" Hazine haftalik limiti ~143,000 birim -> gunluk ~{daycap:,.0f} birim (milyon-sim varsayimi). Ortu islemi maliyeti = 120 birim x %0.5 = 0.6 birim/islem")
L(f" {'kullanici':>10} {'tur 1dk':>22} {'tur 5dk':>22} {'tur 15dk':>22} | 20 emir icin medyan bekleme")
for N in (1_000,10_000,100_000,1_000_000):
    lam=N*0.3/1440  # emir/dk
    cells=[]
    for Lm in (1,5,15):
        rounds=1440//Lm
        X=rng.poisson(lam*Lm,size=20000)
        covers=np.maximum(0,need-X).mean()*rounds
        cost=covers*size*spread
        cells.append(f"{covers:>8,.0f} ortu {cost/daycap:>5.0%}")
    wait=need/lam if lam>0 else 1e9
    L(f" {N:>10,} "+" ".join(f"{c:>22}" for c in cells)+f" | {wait:7.1f} dk")
L(" (ortu islemi sayisi/gun ve maliyetin gunluk hazine limitine orani). Anlik karar + ortu islemi, kucuk ekosistemde hazine limitinin cogunu yer; %100 uzeri = limit yetmez.")
# ---------- C) fake token ----------
L("\nC) SAHTE DK: yeni alicilarin sahtesine kanma beklentisi (SADECE VARSAYIM; gercek davranis olculmedi)")
for v in (0.05,0.2,0.5,0.8):
    newb=1000; p_see=0.3; loss=100
    victims=newb*p_see*(1-v)
    L(f"   dogrulama orani {v:>4.0%}: gunde {newb} yeni alicidan ~{victims:5.0f} kisi sahteyle karsilasip dogrulamiyor (~{victims*loss:,.0f} birim zarar)")
L(" Hash tek basina dogrulama orani artirmaz; resmi mint adresinin tek kaynaktan (site, cuzdan/listeleme dogrulamasi) gorunmesi gerekir.")
