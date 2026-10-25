"""'10 -> 1 dolar' stres testi (normalize: baslangic fiyat=1). MODEL: varsayimlar olculmus veri degildir.
Birimler: toplam arz=1. Havuz token derinligi = d (arzin %'si), havuz stabil = d (fiyat 1).
Sahipler: balina %5, perakende %25 (panik satar), iceridekiler %12 (kilitsizse tek seferde acilir).
Dis alicilar: gunluk kapasite = %3 havuz stabili, fiyat adil degerin altindaysa 4 kata kadar artar.
Savunmalar: cap=hacim-tetikli gunluk sinir (balina+iceridekiler; perakende kucuk satis muaf), lock=iceridekiler 36 ay lineer, bb=geri alim nakdi (havuz stabilinin %10'u, p<0.7*30g ort., gunluk <=%1,5)."""
import random, math, statistics as st
from collections import deque
DAYS=240; T0=100
def run(seed,d,attack,cap,lock,bb):
    r=random.Random(seed); X=d; Y=d; F=1.0
    hist=deque([1.0]*31,maxlen=31)
    ret=0.25; whale=0.05 if 'W' in attack else 0.0
    ins=0.12 if 'I' in attack else 0.0
    ins_sold_rate=ins/1080 if lock else None
    cash=0.10*d if bb else 0.0
    wq=0.0; iq=0.0; mn=1.0
    for day in range(DAYS):
        p=Y/X; a30=sum(list(hist)[-30:])/30; dd7=max(0.0,1-p/hist[-8])
        F*=math.exp(r.gauss(0,0.025))
        if 'D' in attack and T0<=day<T0+10: F*=0.94   # talep cokusu: ~%46 dusus
        def buy(usd):
            nonlocal X,Y
            if usd>0: nY=Y+usd; X=X*Y/nY; Y=nY
        def sell(tok):
            nonlocal X,Y
            if tok<=0: return
            nX=X+tok; out=Y-X*Y/nX; Y-=out; X=nX
        allow=cap*X if cap else None; used=0.0
        def scap(tok):
            nonlocal used
            if allow is None: sell(tok); return tok
            t=min(tok,max(0.0,allow-used)); used+=t; sell(t); return t
        # saldirilar
        if day==T0:
            wq=whale
            if ins and not lock: iq=ins
        if ins and lock: iq=min(iq+ins_sold_rate*1.0,ins); 
        if iq>0:
            want=iq if not lock else iq
            if not lock: want=min(iq,0.5*ins/5*1.0) if day>=T0 else 0.0
            else: want=iq if day>=T0 else 0.0
            if lock and day<T0: want=0.0
            t=scap(want); iq-=t
        if wq>0 and day>=T0:
            t=scap(wq); wq-=t
        # panik (perakende, muaf)
        q=min(0.05,0.002+0.25*max(0.0,dd7-0.10)); s=ret*q; ret-=s; sell(s)
        # dis alicilar
        gap=max(0.0,F/p-1)
        cap_buy=0.03*d*min(4.0,1+3*gap) if F>p else 0.0
        buy(cap_buy*min(1.0,gap*10)) if gap>0 else None
        # geri alim
        if bb and cash>0 and p<0.7*a30:
            sp=min(cash,0.015*Y); cash-=sp; buy(sp)
        # perakende geri doner (alici): fiyat adil degere yakinsa dusuk
        hist.append(Y/X); mn=min(mn,Y/X)
    return mn,Y/X
def ev(d,attack,cap,lock,bb,N=120):
    R=[run(s,d,attack,cap,lock,bb) for s in range(N)]
    return 100*st.median(x[0] for x in R),100*sum(x[1]<0.3 for x in R)/N
if __name__=='__main__':
    DEF=[('korumasiz',None,False,False),('sinir %2',0.02,False,False),('sinir+kilit',0.02,True,False),('sinir+kilit+geri alim',0.02,True,True)]
    ATT=[('A: balina %5 satar','W'),('B: balina + iceridekiler kilitsiz %12','WI'),('C: talep cokusu + panik','D'),('D: B + C birlikte','WID')]
    for an,a in ATT:
        print(f'\n== {an} == (en dusuk fiyat % baslangica gore | 240 gun sonunda <%30 olan kosu %)')
        print('derinlik/arz   '+'  '.join(f'{n:>24}' for n,_,_,_ in DEF))
        for dd in (0.005,0.01,0.02,0.05):
            row=[]
            for n,c,l,b in DEF:
                lo,col=ev(dd,a,c,l,b); row.append(f'{lo:5.0f}% | coker %{col:3.0f}'.rjust(24))
            print(f'  %{dd*100:<4.1f}        '+'  '.join(row))
