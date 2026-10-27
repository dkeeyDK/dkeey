import math
def L(s=""): print(s)
STEP=0.02; LG=math.log(1+STEP); SPREAD=0.005; FEE=0.003; TOTBINS=127
X0=63.0                         # ladder supply sold at start, in bins (B=1 token-bin)
CAP=0.02*TOTBINS                # daily sell cap, 2% of ladder supply
price=lambda x:(1+STEP)**x
cum=lambda x:((1+STEP)**x-1)/LG  # value of tokens up to position x
def run(d, dump_frac, design, arb_share):
    M=price(X0)*X0; R=d*M; Y=R/price(X0)          # external AMM, same start price as ladder
    x=X0; reserve=cum(X0); res0=reserve
    p_amm0=R/Y
    s=dump_frac*X0                                # holders dump on AMM (no cap there)
    got=R-R*Y/(Y+s*(1-FEE)); Y+=s*(1-FEE); R-=got   # AMM trade
    # note: dumped tokens remain in holders' hands no more; they left circulation into AMM
    dump_price=got/s
    arb_profit=0.0; out=[]
    cap_arb=CAP*(1.0 if design=="fifo" else arb_share)
    for day in range(1,31):
        # arbitrage: buy on AMM, sell to ladder while AMM price < ladder bid
        used=0.0
        def gap(q):
            Yn=Y-q; Rn=R*Y/Yn if Yn>0 else 1e18
            return Rn/Yn - price(x-q)*(1-SPREAD)       # AMM price after - ladder bid after
        cap_left=cap_arb
        if R/Y < price(x)*(1-SPREAD) and cap_left>0:
            lo,hi=0.0,min(cap_left,0.999*Y,x)
            if gap(hi)<0: q=hi
            else:
                for _ in range(60):
                    m=(lo+hi)/2
                    if gap(m)<0: lo=m
                    else: hi=m
                q=lo
            Yn=Y-q; Rn=R*Y/Yn
            cost=(Rn-R)/(1-FEE)                       # quote paid to AMM incl. fee approx
            proceeds=(cum(x)-cum(x-q))*(1-SPREAD)
            arb_profit+=proceeds-cost
            reserve-=proceeds; x-=q; Y=Yn; R=R+cost*(1-FEE)
        if day in (1,7,30):
            out.append((day,(R/Y)/p_amm0,price(x)/price(X0),reserve/res0,arb_profit))
    return dump_price/p_amm0,out
L("MERDIVEN + DIS AMM HAVUZU: panik satisi (holder'larin bir kismi tamamini DIS havuza satar; dis havuzda cikis siniri yok)")
L("Birimler goreli: baslangic fiyati=1, merdiven yarisi dolu. Arbitrajci: dis havuzdan ucuza alip merdivene (bid=fiyat x 0.995) satar; gunluk limit 2% arz.")
L("FIFO limit = arbitrajci limitin tamamini kullanabilir. ORANTILI limit = limit, gun basindaki varliga gore paylasilir; arbitrajcinin payi ~0 (araya giren varlik ertesi gun sayilir).\n")
L(f" {'derinlik':>8} {'panik':>5} {'limit':>9} {'arb.pay':>7} | {'damp ort.fiyat':>14} | gun | {'dis fiyat':>9} {'merdiven fiyati':>15} {'rezerv kalan':>12} {'arbitrajci kari':>15}")
for d in (0.05,0.20):
    for f in (0.10,0.30):
        for design,share in (("fifo",1.0),("prorata",0.0),("prorata",0.05)):
            dp,out=run(d,f,design,share)
            for (day,pa,pl,rr,ap) in out:
                L(f" {d:>8.0%} {f:>5.0%} {design:>9} {share:>7.0%} | {dp:>14.2f} | {day:>3} | {pa:>9.2f} {pl:>15.2f} {rr:>12.0%} {ap:>15.1f}")
L("\nOkuma: 'merdiven fiyati' ve 'rezerv kalan' baslangica gore oran; 'arbitrajci kari' birim=baslangic fiyati x token-bin.")
