"""Adil karsilastirma: balina satisinda KUCUK YATIRIMCI zarari. Model/varsayim; olculmus veri degil.
Olcu 1: kucuk yatirimci gerceklesen satis fiyat kaybi (balinasiz kosuya gore EK, puan): satis gunu..+10 gun perakende satislarinin hacim agirlikli ort. fiyati / on-fiyat.
Olcu 2: balinanin yarattigi EK azami fiyat dususu (puan).
Mekanizma maliyeti: ek likidite (USD), hazine nakdi (USD), kilit/sinir kisiti."""
import random, math, statistics as st
from collections import deque
DAYS=240
def run(seed,scen,whale,Y0=100000.0,cap_mode='none',cap=0.05,th=2.0,bb_cash=0.0,vest=1):
    r=random.Random(seed); p0=0.05; Y=Y0; X=Y/p0; F=p0
    hist=deque([p0]*31,maxlen=31); cash=bb_cash; dump=120
    queue=0.0; pre=None; mn=1e9
    rs_val=0.0; rs_tok=0.0; whale_daily=None
    for d in range(DAYS):
        p=Y/X; a30=sum(list(hist)[-30:])/30; r7=max(-0.5,min(1.0,p/hist[-8]-1)); r3=p/hist[-4]-1
        drift=-0.012 if (scen=='decline' and 95<=d<120) else 0.0
        F*=math.exp(r.gauss(drift,0.03))
        def buy(usd):
            nonlocal X,Y
            if usd<=0: return
            nY=Y+usd; X=X*Y/nY; Y=nY
        def sell(tok):
            nonlocal X,Y
            if tok<=0: return 0.0
            nX=X+tok; out=Y-X*Y/nX; Y=Y-out; X=nX; return out
        allow=None
        if cap_mode=='price' and p>th*a30: allow=cap*X
        elif cap_mode=='volume': allow=cap*X
        used=[0.0]
        def scap(tok):
            if allow is None: return tok,sell(tok)
            t=min(tok,max(0.0,allow-used[0])); used[0]+=t; return t,sell(t)
        base=800*math.exp(r.gauss(0,0.5))
        buy(base*(1+3*max(0,r7)))
        st_=(base*0.7*(1+4*max(0,-r7)))/p
        t,u=scap(st_)
        if dump<=d<dump+10: rs_tok+=t; rs_val+=u
        if r3>0: buy(0.004*Y)
        else: scap(0.004*Y/p)
        p=Y/X; k=X*Y; pt=p*(1+0.05*(F/p-1))
        if pt>p: buy(math.sqrt(k*pt)-Y)
        elif pt<p: sell(math.sqrt(k/pt)-X)
        if d==dump:
            pre=sum(list(hist)[-3:])/3; queue=whale*2000000.0
            whale_daily=queue/vest
        if queue>0 and d>=dump:
            want=min(queue,whale_daily); t,u=scap(want); queue-=t
        p=Y/X
        if bb_cash>0 and cash>0 and p<0.7*a30:
            sp=min(cash,0.015*Y); cash-=sp; buy(sp)
        hist.append(Y/X)
        if pre is not None and d>=dump: mn=min(mn,Y/X)
    vw=(rs_val/rs_tok) if rs_tok>0 else pre
    return 100*(1-vw/pre), 100*(1-mn/pre)
BASE={}
def base(seed,scen,Y0):
    k=(seed,scen,Y0)
    if k not in BASE: BASE[k]=run(seed,scen,0.0,Y0)
    return BASE[k]
def ev(scen,whale,N=250,**kw):
    Y0=kw.get('Y0',100000.0); a=[];b=[]
    for s in range(N):
        x=run(s,scen,whale,**kw); y=base(s,scen,Y0)
        a.append(x[0]-y[0]); b.append(x[1]-y[1])
    return st.mean(a),st.mean(b)
if __name__=='__main__':
    cfg=[('0 Hicbir sey (yalniz havuz 100k)',dict(),0,'-'),
     ('1a 2 kat derin havuz',dict(Y0=200000.0),100000,'+100k USD likidite'),
     ('1b 4 kat derin havuz',dict(Y0=400000.0),300000,'+300k USD likidite'),
     ('2a Fiyat-tetikli %5 sinir (2x)',dict(cap_mode='price',cap=0.05,th=2.0),0,'0'),
     ('2b Hacim-tetikli %3/gun sinir',dict(cap_mode='volume',cap=0.03),0,'0, kisit: tum satislara'),
     ('2c Hacim-tetikli %1/gun sinir',dict(cap_mode='volume',cap=0.01),0,'0, kisit: tum satislara'),
     ('3a Geri alim, nakit 5k',dict(bb_cash=5000.0),5000,'5k USD nakit'),
     ('3b Geri alim, nakit 30k',dict(bb_cash=30000.0),30000,'30k USD nakit'),
     ('4a Kilit: 10 gunde yayilan satis',dict(vest=10),0,'balina 10 gun kilitli'),
     ('4b Kilit: 30 gunde yayilan satis',dict(vest=30),0,'balina 30 gun kilitli')]
    for scen,whale,title in (('calm',0.25,'Sakin piyasa, balina %25'),('calm',0.75,'Sakin piyasa, balina %75'),('decline',0.75,'Dususteki piyasa, balina %75')):
        print(f'\n== {title} == (EK zarar, puan: kucuk yat. satis kaybi | azami dusus)')
        for name,kw,cost,note in cfg:
            a,b=ev(scen,whale,**kw); print(f'{name:36} {a:6.1f} | {b:6.1f}   maliyet: {note}')
