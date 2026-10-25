"""Oneri paketinin taramasi: gunluk satis siniri (havuz token derinliginin %2'si) ve uc uygulama bicimi. MODEL/varsayim.
Mod 'none': sinir yok. 'prorata': sinir TUM satislara, her tokene esit cikis hakki (cuzdan sayisi onemsiz, bolme ise yaramaz).
'exempt': perakende muaf, balina sinirli; balinanin 'split' orani kadari kucuk hesaplara bolunup muaf kulvardan satilir (kimliksiz sistemde engellenemez).
Olcu: en dusuk fiyat | perakende gun0-20 hacim-agirlikli satis fiyati | perakende satis doluluk % | 240.gun fiyat."""
import random, math, statistics as st
from collections import deque
DAYS=240; T0=100
def run(seed,d,att,mode,split=0.0,cap=0.02):
    r=random.Random(seed); X=d; Y=d; F=1.0; hist=deque([1.0]*31,maxlen=31)
    ret=0.25; wq=0.05 if 'W' in att else 0.0; mn=1.0
    rv=rt=rw=0.0
    for day in range(DAYS):
        p=Y/X; dd7=max(0.0,1-p/hist[-8])
        F*=math.exp(r.gauss(0,0.025))
        if 'D' in att and T0<=day<T0+10: F*=0.94
        def buy(u):
            nonlocal X,Y
            if u>0: nY=Y+u; X=X*Y/nY; Y=nY
        def sell(t):
            nonlocal X,Y
            if t<=0: return 0.0
            nX=X+t; out=Y-X*Y/nX; Y-=out; X=nX; return out
        q=min(0.05,0.002+0.25*max(0.0,dd7-0.10)); rq=ret*q
        wreq=wq if day>=T0 else 0.0
        allow=cap*X if mode!='none' else None
        if mode=='none': fr,fw=rq,wreq
        elif mode=='prorata':
            R=rq+wreq; f=1.0 if R<=allow else allow/R; fr,fw=rq*f,wreq*f
        elif mode=='exempt':
            sp=wreq*split; fw=min(wreq-sp,allow)+sp; fr=rq
        u=sell(fr); ret-=fr; wq-=fw; sell(fw)
        if T0<=day<T0+20: rv+=u; rt+=fr; rw+=rq
        gap=max(0.0,F/(Y/X)-1)
        if gap>0 and F>Y/X: buy(0.03*d*min(4.0,1+3*gap)*min(1.0,gap*10))
        hist.append(Y/X); mn=min(mn,Y/X)
    vw=(rv/rt) if rt>0 else 1.0
    return mn,vw,(rt/rw if rw>0 else 1.0),Y/X
def ev(d,att,mode,split=0.0,N=100):
    R=[run(s,d,att,mode,split) for s in range(N)]
    m=lambda i:100*st.median(x[i] for x in R)
    return m(0),m(1),m(2),m(3)
if __name__=='__main__':
    CF=[('sinir yok','none',0),('adil cikis (orantili, tum satislar)','prorata',0),('muafiyet, bolme YOK','exempt',0.0),('muafiyet, balina %50 boler','exempt',0.5),('muafiyet, balina %100 boler','exempt',1.0)]
    for an,a in (('A: balina %5 satar','W'),('D: talep cokusu + panik + balina','WD')):
        print(f'\n== {an} ==  (en dusuk fiyat % | perakende satis fiyati % | perakende satis doluluk % | 240.gun fiyat %)')
        for dd in (0.01,0.02,0.05):
            print(f' derinlik %{dd*100:.0f}')
            for n,m,s in CF:
                v=ev(dd,a,m,s); print(f'   {n:36} {v[0]:5.0f} | {v[1]:5.0f} | {v[2]:5.0f} | {v[3]:5.0f}')
