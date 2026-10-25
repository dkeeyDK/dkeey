"""Denge/sonum simulasyonu: AMM havuzu + perakende + bot + balina + gercek-deger arbitraji.
Sonumleyiciler: (1) %cap toplu satis siniri (yalniz fiyat 30g ort.nin 2 kati iken), (2) butceli geri alim (fiyat 30g ort.nin %70 alti), (3) gelir (enerji) = ucret payi gelir kasasina.
Bu bir MODELDIR: parametreler varsayimdir, olculmus veri degildir."""
import random, math, statistics as st
DAYS=300
def run(seed,scen,cap,bb,whale=0.25,bots=True,rev=True):
    r=random.Random(seed)
    Y=100000.0; p0=0.05; X=Y/p0; X0=X
    F=p0; price=[p0]*31; surplus=0.0; whale_tok=whale*X0
    dump_day=120 if scen=='crash' else 90
    pump=(40,70) if scen=='spike' else None
    queue=0.0; panic=0.0; retail_sell_total=0.0
    pre=None; mn=1e9; rec=None
    for d in range(DAYS):
        p=Y/X; avg30=st.mean(price[-30:]); r7=max(-0.5,min(1.0,p/price[-8]-1)); r3=p/price[-4]-1
        # gercek deger: GBM + pump
        F*=math.exp(r.gauss(0,0.03))
        if pump and pump[0]<=d<pump[1]: F*=1.045
        vol_usd=0.0
        def buy(usd):
            nonlocal X,Y,vol_usd
            if usd<=0: return 0.0
            newY=Y+usd; newX=X*Y/newY; out=X-newX; X=newX; Y=newY; vol_usd+=usd; return out
        def sell(tok):
            nonlocal X,Y,vol_usd
            if tok<=0: return 0.0
            newX=X+tok; newY=X*Y/newX; out=Y-newY; X=newX; Y=newY; vol_usd+=out; return out
        # sat. siniri: fiyat 30g ort 2 kati ustundeyse gunluk toplu satis tavani
        allow=None
        if cap is not None and p>2*avg30: allow=cap*X
        used=0.0
        def sell_cap(tok):
            nonlocal used
            if allow is None: sell(tok); return tok
            t=min(tok,max(0.0,allow-used)); used+=t; sell(t); return t
        # perakende
        base=800*math.exp(r.gauss(0,0.5))
        mult=6 if (pump and pump[0]<=d<pump[1]) else 1
        buy(base*mult*(1+3*max(0,r7)))
        sell_t=(base*0.7*(1+4*max(0,-r7)))/p
        s_done=sell_cap(sell_t); 
        if dump_day<=d<dump_day+10: panic+=s_done*p; retail_sell_total+=sell_t*p
        # botlar (momentum)
        if bots:
            if r3>0: buy(0.004*Y)
            else: sell_cap(0.004*Y/p)
        # arbitraj: bosluk %5/gun kapanir
        p=Y/X
        gap=F/p-1; k=X*Y; pt=p*(1+0.05*gap)
        if pt>p: buy(math.sqrt(k*pt)-Y)
        elif pt<p: sell(math.sqrt(k/pt)-X)
        # balina
        if d==dump_day: queue=whale*X; pre=st.mean(price[-3:])
        if queue>0 and d>=dump_day:
            sold=sell_cap(queue); queue-=sold
        # geri alim
        p=Y/X
        if bb and p<0.7*avg30 and surplus>0:
            spend=min(surplus,bb*Y); surplus-=spend
            got=buy(spend); X-=got*0  # alinan token yakilir (havuzdan zaten cikti)
        # gelir (enerji)
        if rev: surplus+=0.0015*vol_usd
        price.append(Y/X)
        if pre is not None and d>=dump_day:
            mn=min(mn,price[-1])
            if rec is None and d>dump_day+1 and price[-1]>=0.9*pre: rec=d-dump_day
    dd=1-mn/pre if pre else 0
    return dd,rec,(panic/retail_sell_total if retail_sell_total else 0),st.pstdev([math.log(max(price[i],1e-12)/max(price[i-1],1e-12)) for i in range(32,len(price))])
def agg(scen,cap,bb,whale=0.25,N=300,**k):
    R=[run(s,scen,cap,bb,whale,**k) for s in range(N)]
    dd=st.median(x[0] for x in R); rc=[x[1] for x in R if x[1] is not None]
    return dict(dd=100*dd,rec=(st.median(rc) if rc else None),unrec=100*(1-len(rc)/N),vol=100*st.mean(x[3] for x in R))
if __name__=='__main__':
    rows=[('korumasiz',None,0),('yalniz %5 sinir',0.05,0),('yalniz geri alim %1,5',None,0.015),('ikisi birlikte',0.05,0.015)]
    for scen in ('crash','spike'):
        for whale in (0.25,0.75):
            print(f'\n== Senaryo: {scen}, balina havuz tokeninin %{int(whale*100)}\'ini satiyor ==')
            for n,c,b in rows:
                a=agg(scen,c,b,whale)
                print(f"{n:24} max dusus %{a['dd']:5.1f} | toparlanma gun {a['rec'] if a['rec'] is None else int(a['rec'])!s:>4} | toparlanamayan %{a['unrec']:5.1f} | gunluk oynaklik %{a['vol']:.2f}")
    print('\n== Gelir (enerji) kapali, geri alim acik, spike/%25 ==')
    for rev in (True,False):
        a=agg('spike',0.05,0.015,0.25,rev=rev); print(f"gelir={'var' if rev else 'yok'} max dusus %{a['dd']:.1f} toparlanamayan %{a['unrec']:.1f}")
    print('\n== Botlar (safsizlik) kapali/acik, korumasiz vs tam korumali, spike/%25 ==')
    for bots in (False,True):
        for n,c,b in (('korumasiz',None,0),('tam',0.05,0.015)):
            a=agg('spike',c,b,0.25,bots=bots); print(f"botlar={'var' if bots else 'yok'} {n:9} max dusus %{a['dd']:.1f} oynaklik %{a['vol']:.2f}")
    print('\n== Satis siniri taramasi (spike, balina %75, geri alim kapali) ==')
    for c in (None,0.02,0.05,0.10,0.20):
        a=agg('spike',c,0,0.75); print(f"sinir {('yok' if c is None else '%'+str(int(c*100))):>5}: max dusus %{a['dd']:.1f} toparlanma {a['rec']} toparlanamayan %{a['unrec']:.1f}")
