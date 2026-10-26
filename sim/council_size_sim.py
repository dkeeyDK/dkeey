"""Meclis buyuklugu (tavan) analizi. Modeldir: varsayimlara bagli, kanit degil.
Olculenler: (1) kararin dogru cikma olasiligi (uyeler tamamen bagimsiz degil: korelasyon rho),
(2) yetersayi: katilim orani dusukken karar cikabilir mi, (3) ele gecirme maliyeti (koltuk basi maliyet c ile dogrusal),
(4) oy basina maliyet (LLM cikarimi, dogrusal)."""
import math
def lbeta(a,b): return math.lgamma(a)+math.lgamma(b)-math.lgamma(a+b)
def bb_pmf(k,n,a,b):  # beta-binomial
    return math.exp(math.lgamma(n+1)-math.lgamma(k+1)-math.lgamma(n-k+1)+lbeta(k+a,n-k+b)-lbeta(a,b))
def majority_acc(n,q,rho):
    if rho<=0:
        return sum(math.comb(n,k)*q**k*(1-q)**(n-k) for k in range(n//2+1,n+1))
    s=1/rho-1; a=q*s; b=(1-q)*s
    return sum(bb_pmf(k,n,a,b) for k in range(n//2+1,n+1))
def quorum_ok(n,presence,quorum=0.5):
    need=math.floor(n*quorum)+1
    return sum(math.comb(n,k)*presence**k*(1-presence)**(n-k) for k in range(need,n+1))
Ns=[25,50,75,101,151,201,301,501]
print("Meclis buyuklugu analizi (model). q=her uyenin tek basina dogru karar olasiligi")
for q,rho in ((0.70,0.05),(0.70,0.15),(0.70,0.30),(0.60,0.15)):
    print(f"\nq={q} korelasyon rho={rho}  (sinir deger = N->sonsuz)")
    s=1/rho-1; a=q*s; b=(1-q)*s
    # sonsuz N siniri: P(p>0.5) beta dagilimi icin
    lim=None
    import math as m
    # sayisal integral
    steps=20000; tot=0; cdf=0
    for i in range(1,steps):
        x=i/steps; pdf=m.exp((a-1)*m.log(x)+(b-1)*m.log(1-x)-lbeta(a,b)); tot+=pdf/steps
        if x>0.5: cdf+=pdf/steps
    lim=cdf/tot
    print("  N    dogru%   sinira_kalan   oy basina maliyet(x50)  ele gecirme maliyeti(x50)")
    for n in Ns:
        acc=majority_acc(n,q,rho)
        print(f"  {n:3d}  {100*acc:6.2f}   {100*(lim-acc):6.2f}         {n/50:5.1f}x                  {n/50:5.1f}x")
print("\nYetersayi (>%50 katilim gerekir): uye basina katilim olasiligi p")
for p in (0.95,0.90,0.80,0.70):
    print(f"  p={p}: "+"  ".join(f"N={n}:{100*quorum_ok(n,p):.1f}%" for n in (25,50,101,201,501)))
