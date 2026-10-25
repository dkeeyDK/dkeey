import math
P0=10.0
TVL0=40e6                     # mevcut havuz toplam degeri (varsayim): 20M stable + 2M DK
L0=TVL0/(2*math.sqrt(P0))
def L_full(V): return V/(2*math.sqrt(P0))
def L_conc(V,r):
    pa=P0/r; pb=P0*r
    return V/(2*math.sqrt(P0)-P0/math.sqrt(pb)-math.sqrt(pa))
V=100e6; R=2.0
Lf=L_full(V); Lc=L_conc(V,R); pa=P0/R
def drop(spend_usd, mode):
    s=spend_usd/P0                      # satilan token (baslangic fiyatindan)
    # fiyat asagi: sqrt(P) = ... 1/sqrtP' = 1/sqrtP + s/L ; kismi LP aralik disina cikinca devre disi
    def price_after(Ltot_fn):
        # adim adim: once LP dahil, aralik siniri pa'ya ulasinca LP'siz devam
        pass
    if mode=='base': L=L0; inv=1/math.sqrt(P0)+s/L; return 1-(1/inv**2)/P0
    if mode=='full': L=L0+Lf; inv=1/math.sqrt(P0)+s/L; return 1-(1/inv**2)/P0
    if mode=='exit': L=L0; inv=1/math.sqrt(P0)+s/L; return 1-(1/inv**2)/P0
    if mode=='conc':
        L=L0+Lc; cap_tokens=L*(1/math.sqrt(pa)-1/math.sqrt(P0))
        if s<=cap_tokens: inv=1/math.sqrt(P0)+s/L; return 1-(1/inv**2)/P0
        rest=s-cap_tokens                 # aralik bitti: LP tokenleri doldu, sadece taban havuz kaldi
        inv=1/math.sqrt(pa)+rest/L0; return 1-(1/inv**2)/P0
print("Balina satisi (USD) -> fiyat dususu. Mevcut havuz 40M USD TVL; ekstra LP 100M USD")
print(f"{'satis':>8} | {'LP yok':>7} | {'LP tam aralik':>13} | {'LP ±2x ajan':>11} | {'LP once cikarsa':>15}")
for sp in (2e6,10e6,25e6,50e6,100e6):
    print(f"{sp/1e6:6.0f}M | %{drop(sp,'base')*100:5.0f}  | %{drop(sp,'full')*100:10.0f}  | %{drop(sp,'conc')*100:8.0f}  | %{drop(sp,'exit')*100:11.0f}")
# Ucret geliri (varsayim: gunluk hacim 20M USD, ucret %0.3) ve kayip (impermanent loss)
vol=20e6; fee=0.003
for name,L in (("tam aralik",Lf),("±2x ajan",Lc)):
    share=L/(L+L0); inc=vol*fee*share*365
    print(f"\n{name}: havuz payi %{share*100:.0f}, yillik ucret ~{inc/1e6:.1f}M USD = %{inc/V*100:.1f} (aralikta kalirsa)")
def il(k,L,r):
    P=P0*k
    if r is None:
        return 1-2*math.sqrt(k)/(1+k)
    pa=P0/r; pb=P0*r
    x0=L*(1/math.sqrt(P0)-1/math.sqrt(pb)); y0=L*(math.sqrt(P0)-math.sqrt(pa))
    Pc=min(max(P,pa),pb)
    x=L*(1/math.sqrt(Pc)-1/math.sqrt(pb)); y=L*(math.sqrt(Pc)-math.sqrt(pa))
    hold=x0*P+y0; val=x*P+y
    return 1-val/hold
print("\nFiyat k kati olunca LP'nin sadece tutmaya gore kaybi (impermanent loss):")
for k in (0.5,0.8,1.25,2.0):
    print(f"  k={k}: tam aralik %{il(k,None,None)*100:4.1f} | ±2x %{il(k,Lc,R)*100:4.1f}")
