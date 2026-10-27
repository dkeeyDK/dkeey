import numpy as np
from math import comb
rng=np.random.default_rng(11)
def L(s=""): print(s)
DAYCAP=143_000/7; BUDGET=0.25*DAYCAP; SPREAD=0.005; ROUNDS=1440; ORD_PER_USER=0.3
def level(s):
    p=1/max(s,1); return ("YUKSEK" if s>=20 else "ORTA" if s>=5 else "DUSUK"), p
L("PART 1) GIZLILIK SEVIYESI EGRISI - anlik islem (tur 60 sn, kimse beklemiyor). Olcu: bir islemin karsi tarafini bilmek icin saldirganin aday sayisi s (taraf basina).")
L(f" Ortu butcesi = gunluk hazine limitinin %25'i = {BUDGET:,.0f} birim/gun. Ortu islemi cifti = alici+satici; maliyet = 2 x boyut x %0.5.")
L(" DUNYA H: tutarlar gizli (ortu 5 birim olabilir). DUNYA V: tutarlar gorunur (ortu gercek dagilimi taklit etmeli, ort. 120 birim; ayrica saldirgan tutarla aday eler).")
# visible-amount matching: simulate candidate-set shrinkage by amount filter
def vis_filter(k,tol=0.02,trials=400):
    out=[]
    for _ in range(trials):
        a=rng.lognormal(np.log(60),1.5,k)   # alici tutarlari
        b=a.copy()
        b=b[rng.permutation(k)]             # satici tutarlari (ayni kume, karisik); gercek eslesme ayni tutar
        i=rng.integers(k)
        cand=np.sum(np.abs(np.log(b)-np.log(a[i]))<tol)
        out.append(cand)
    return np.mean(out)
L(f" {'kullanici':>9} {'gercek/taraf':>12} | H: ortu cifti/tur  s   seviye  P(eslestirme) | V: ortu/tur  s_ham  s_tutarli  seviye  P(eslestirme)")
rows=[]
for N in (100,1_000,5_000,10_000,50_000,100_000,1_000_000):
    real=N*ORD_PER_USER/ROUNDS/2   # taraf basina gercek emir/tur
    # H
    cost_pair_H=2*5*SPREAD; c_H=min(BUDGET/ROUNDS/cost_pair_H, max(0,30-real))
    sH=real+c_H; lvH,pH=level(sH)
    # V
    cost_pair_V=2*120*SPREAD; c_V=min(BUDGET/ROUNDS/cost_pair_V, max(0,30-real))
    sV_raw=real+c_V
    k=max(2,int(round(sV_raw))); cons=vis_filter(min(k,2000)) if k<=2000 else 1+ (k-1)*0.0
    lvV,pV=level(cons)
    L(f" {N:>9,} {real:>12.2f} | {c_H:>14.1f} {sH:>5.1f} {lvH:>8} {pH:>10.1%} | {c_V:>9.1f} {sV_raw:>7.1f} {cons:>10.1f} {lvV:>8} {pV:>10.1%}")
L(" Okuma: H dunyasinda ortu ucuz oldugu icin kucuk kullanici sayisinda da seviye hizla yukselir. V dunyasinda tutar gorunur oldugundan aday kumesi tutar filtresiyle yaklasik 1-2'ye iner: kalabalik ne olursa olsun bulaniklik YOK denecek kadar az.")
L(" (Not: V'de tutar parcalayarak/yuvarlayarak bunu azaltma fikri ayri bir arastirma; burada modellenmedi.)")

L("\nPART 2) DEVNET SONUCU SENARYOLARI: gizli tutar/transfer hook DEX'te kabul edilirse (H) ya da edilmezse (V). Dis DEX hacim payi x'e gore korunan hacim.")
L(" Korunan hacim = ic merdiven hacmi (H ya da V seviyesiyle) ; dis DEX hacmi gorunur (bulanik degil).")
for x in (0.0,0.3,0.7):
    L(f"   dis DEX hacim payi {x:>4.0%}: H dunyasi: ic hacim %{(1-x)*100:.0f} bulanik, dis %{x*100:.0f} acik | V dunyasi: bulaniklik pratikte yok (tutar eslesmesi)")
L(" Hook DEX islemini engellerse (dis listeleme olmaz): likidite yalniz ic merdivende; hacim payi dis=0. Bunu hacim/likidite olarak modelleyemedim (gercek veri yok).")

L("\nPART 3) CEKIRDEK ELE GECIRILIRSE: eslesmeyi gormek icin esik t=6/8 koltuk gerekir (5 LLM + 3 kural koltugu). Saldirgan eslesmeyi gorme olasiligi:")
def binom_ge(n,k,p): return sum(comb(n,j)*p**j*(1-p)**(n-j) for j in range(k,n+1))
L(f" {'p (koltuk basi)':>16} | q=ortak LLM kor nokta: 0%    1%     5%   (1/8 koltuk tek basina, her koltuk bagimsiz)")
for p in (0.02,0.05,0.10,0.20,0.30):
    cells=[]
    for q in (0,0.01,0.05):
        # ortak olay: 5 LLM duser, kalan 3 kural koltugundan en az 1 bagimsiz p ile
        P_common=1-(1-p)**3
        P_ind=binom_ge(8,6,p)
        cells.append(q*P_common+(1-q)*P_ind)
    L(f" {p:>16.0%} | "+"  ".join(f"{c:>8.3%}" for c in cells))
L(" Okuma: LLM'lerin ortak kor noktasi (q) bagimsiz p'den cok daha buyuk risk; kural koltuklari bunu 3 koltuk ile sinirlar. Eslesmeyi gorme = kullanici gizliligi kaybi (para kaybi degil).")

L("\nPART 4) SALDIRGAN ORTU ISLEMLERINI AYIKLAYABILIRSE (dogruluk a): etkin aday kumesi = gercek + (1-a) x ortu")
L(f" {'a':>6} | H dunyasi, 10,000 kullanici: s, P  | 100,000 kullanici: s, P")
for a in (0.0,0.5,0.9,0.99):
    cells=[]
    for N in (10_000,100_000):
        real=N*ORD_PER_USER/ROUNDS/2; cost=2*5*SPREAD
        c=min(BUDGET/ROUNDS/cost,max(0,30-real)); s=real+(1-a)*c
        cells.append(f"{s:5.1f} {1/max(s,1):6.1%}")
    L(f" {a:>6.0%} | {cells[0]:>28} | {cells[1]:>26}")
L(" Okuma: ortu islemleri saldirgan tarafindan ayirt edilirse (zamanlama, ucret, cuzdan kumesi) seviye ham gercek kalabalik kadar kalir.")

L("\nPART 5) TEKRAR EDEN ISLEMLER: bir kullanici T islem yapar; en az birinin eslesmesinin bulunma olasiligi 1-(1-p)^T (bagimsiz tur varsayimi)")
for lab,pp in (("YUKSEK (p=5%)",0.05),("ORTA (p=20%)",0.20),("DUSUK (p=50%)",0.5)):
    L(f"   {lab:>14}: T=1 {pp:5.1%} | T=10 {1-(1-pp)**10:5.1%} | T=100 {1-(1-pp)**100:5.1%}")
L(" Okuma: tek islem 'YUKSEK' bulanik olsa bile cok islem yapan kullanicinin en az bir islemi sizar. Bu yuzden vaat: 'bir islemi bulanik', 'hesabin tum gecmisi gizli' DEGIL.")
L("\nMODELLENMEDI: gercek DEX/hook davranisi, zamanlama ve ucret parmak izi, cuzdan kumeleme, gizli tutar kanitlarinin maliyeti (hesaplama/ucret), hukuki kabul.")
