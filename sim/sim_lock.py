import math
P0=10.0; TVL0=40e6; L0=TVL0/(2*math.sqrt(P0))
V=100e6; vol=20e6; fee=0.003
def Lfull(v): return v/(2*math.sqrt(P0))
def drop(L,spend):
    s=spend/P0; inv=1/math.sqrt(P0)+s/L; return 1-(1/inv**2)/P0
def remaining(m,policy):
    # kalan LP orani (baslangicin kati)
    if policy=="serbest": return 0.0 if m>=1 else 1.0          # 1. aydan sonra tamami cikar
    if policy=="1y kilit": return 1.0 if m<=12 else max(0.0,1-0.02*(m-12)) # 2. yil+ ayda %2 cikis
    if policy=="1y kilit hizli": return 1.0 if m<=12 else max(0.0,1-0.05*(m-12))  # ayda %5
print("Ortak ajan 100M USD LP, mevcut havuz 40M USD. Ortagin cikmak istedigi en kotu durum (cikis hakkini hemen kullanir).")
print("Kolon: o ayda 25M USD balina satisinda fiyat dususu | ortagin havuz payi")
print(f"{'ay':>4} | " + " | ".join(f"{p:>22}" for p in ("serbest","1y kilit (ayda %2)","1y kilit (ayda %5)")))
pol=("serbest","1y kilit","1y kilit hizli")
for m in (1,6,12,18,24,30,36,48):
    row=[]
    for p in pol:
        rem=remaining(m,p); L=L0+Lfull(V)*rem
        row.append(f"%{drop(L,25e6)*100:3.0f}  pay %{100*Lfull(V)*rem/L:3.0f}")
    print(f"{m:4d} | " + " | ".join(f"{x:>22}" for x in row))
print("\nOrtagin yillik ucret geliri (hacim 20M USD/gun, ucret %0.3, havuz payi orantili):")
for p in pol:
    inc=[]
    for yr in (1,2,3,4):
        tot=0
        for m in range((yr-1)*12+1,yr*12+1):
            rem=remaining(m,p); L=L0+Lfull(V)*rem; sh=Lfull(V)*rem/L
            tot+=vol*fee*sh*30
        inc.append(tot/1e6)
    print(f"  {p:>16}: yil1 {inc[0]:4.1f}M | yil2 {inc[1]:4.1f}M | yil3 {inc[2]:4.1f}M | yil4 {inc[3]:4.1f}M USD")
print("\nOrtagin orantili pay: yatirdigi / (yatirim + mevcut havuz) = %.0f%% (tam aralik, kilitliyken sabit)"%(100*Lfull(V)/(L0+Lfull(V))))
