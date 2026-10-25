import random, math, statistics as st
BIN=1.0   # milyon DK / kutu
def run(mode, drift, seed, H=24*30, vol=0.012):
    r=random.Random(seed)
    Pe=1.0; Pn=1.0   # dis fiyat, siradaki kutu fiyati
    sold=0; leak=0; value=0; stall=0; bins=0; prev_rate=0; hist=[]
    for h in range(H):
        Pe*=math.exp(drift/24+r.gauss(0,vol))
        hist.append(Pe)
        twap=sum(hist[-6:])/len(hist[-6:])   # 6 saatlik ortalama (manipulasyona dayanikli)
        buys=0
        # arbitrajcilar: kutu fiyati piyasanin altindaysa hizla alir
        while Pn<twap*0.99 and buys<10 and bins<50:
            sold+=BIN; bins+=1; buys+=1
            leak+=BIN*max(0,Pe-Pn); value+=BIN*Pe
            Pn=next_price(mode,Pn,twap,buys)
        # gercek alicilar: fiyat piyasadan en fazla %3 yukarida ise 2 kutu alir
        if Pn<=twap*1.03 and bins<50:
            for _ in range(1):
                if bins>=50: break
                sold+=BIN; bins+=1; buys+=1
                leak+=BIN*max(0,Pe-Pn); value+=BIN*Pe
                Pn=next_price(mode,Pn,twap,buys)
        else: stall+=1
        prev_rate=buys
    return sold, 100*leak/max(value,1e-9), 100*stall/H, Pn/Pe-1
def next_price(mode,Pn,twap,buys):
    if mode=='sabit': return Pn*1.033
    # kontrolcu: g = g0 + k1*(dis/ic-1) + k2*(talep hizi), sinirli [0, %5]
    g=0.015 + 0.5*(twap/Pn-1) + 0.004*(buys-1)
    return Pn*(1+min(0.05,max(0.0,g)))
print("30 gun, 200 deneme. Dis piyasa: yukari, yatay, asagi egilim. Kutu=1M DK")
print(f"{'senaryo':>8} | {'mod':>9} | satilan(M DK) | kacak(deger %) | durma(%saat) | son fiyat/dis fiyat")
for name,dr in (("yukari",0.03),("yatay",0.0),("asagi",-0.03)):
    for mode in ("sabit","kontrolcu"):
        R=[run(mode,dr,s) for s in range(200)]
        print(f"{name:>8} | {mode:>9} | {st.mean(x[0] for x in R):13.0f} | {st.mean(x[1] for x in R):14.2f} | {st.mean(x[2] for x in R):11.1f} | %{100*st.mean(x[3] for x in R):6.1f}")
