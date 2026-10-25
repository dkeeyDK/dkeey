"""Hacim-tetikli gunluk satis sinirinin adaleti: sira (kim once?), kucuk satis muafiyeti, orantili dagitim, bolunmus balina.
Model/varsayim. Olcu: kucuk yatirimci (a) EK gerceklesen fiyat kaybi, (b) satis doluluk orani (istedigi / satabildigi), (c) balina EK azami dusus."""
import random, math, statistics as st
from collections import deque
DAYS=240
def run(seed,mode,cap,whale=0.75,scen='calm'):
    r=random.Random(seed); p0=0.05; Y=100000.0; X=Y/p0; F=p0
    hist=deque([p0]*31,maxlen=31); dump=120; queue=0.0; pre=None; mn=1e9
    rv=0.0; rt=0.0; want=0.0
    for d in range(DAYS):
        p=Y/X; r7=max(-0.5,min(1.0,p/hist[-8]-1)); r3=p/hist[-4]-1
        F*=math.exp(r.gauss(-0.012 if (scen=='decline' and 95<=d<120) else 0.0,0.03))
        def buy(usd):
            nonlocal X,Y
            if usd>0: nY=Y+usd; X=X*Y/nY; Y=nY
        def sell(tok):
            nonlocal X,Y
            if tok<=0: return 0.0
            nX=X+tok; out=Y-X*Y/nX; Y-=out; X=nX; return out
        base=800*math.exp(r.gauss(0,0.5)); buy(base*(1+3*max(0,r7)))
        ret=(base*0.7*(1+4*max(0,-r7)))/p
        bot=0.0 if r3>0 else 0.004*Y/p
        if r3>0: buy(0.004*Y)
        if d==dump: pre=sum(list(hist)[-3:])/3; queue=whale*2000000.0
        wh=queue if (queue>0 and d>=dump) else 0.0
        allow=None if mode=='none' else cap*X
        if allow is None or mode=='exempt_split': fr,fb,fw=ret,bot,wh
        elif mode=='retail_first':
            fr=min(ret,allow); rem=allow-fr; fb=min(bot,rem); rem-=fb; fw=min(wh,rem)
        elif mode=='whale_first':
            fw=min(wh,allow); rem=allow-fw; fr=min(ret,rem); rem-=fr; fb=min(bot,rem)
        elif mode=='prorata':
            R=ret+bot+wh; f=1.0 if R<=allow else allow/R; fr,fb,fw=ret*f,bot*f,wh*f
        elif mode=='exempt_small':   # kucuk satislar (perakende+bot) muaf, balina sinirli
            fr,fb=ret,bot; fw=min(wh,allow)
        u=sell(fr); sell(fb)
        if dump<=d<dump+10: rt+=fr; rv+=u; want+=ret
        p=Y/X; k=X*Y; pt=p*(1+0.05*(F/p-1))
        if pt>p: buy(math.sqrt(k*pt)-Y)
        elif pt<p: sell(math.sqrt(k/pt)-X)
        if fw>0: sell(fw); queue-=fw
        hist.append(Y/X)
        if pre is not None and d>=dump: mn=min(mn,Y/X)
    vw=(rv/rt) if rt>0 else pre
    return 100*(1-vw/pre), 100*(1-mn/pre), (rt/want if want>0 else 1.0)
BASE={}
def ev(mode,cap,whale,scen,N=250):
    a=[];b=[];c=[]
    for s in range(N):
        k=(s,scen)
        if k not in BASE: BASE[k]=run(s,'none',0.0,0.0,scen)
        x=run(s,mode,cap,whale,scen); y=BASE[k]
        a.append(x[0]-y[0]); b.append(x[1]-y[1]); c.append(x[2])
    return st.mean(a),st.mean(b),100*st.mean(c)
if __name__=='__main__':
    for scen in ('calm','decline'):
        print(f'\n== {scen}, balina %75 == (kucuk yat. EK satis kaybi puan | balina EK dusus puan | kucuk yat. satis doluluk %)')
        a=ev('none',0,0.75,scen); print(f"{'sinir yok':34} {a[0]:6.1f} | {a[1]:6.1f} | {a[2]:5.1f}")
        for cap in (0.03,0.01):
            for mode,name in (('retail_first','sira: kucuk once (iyimser)'),('whale_first','sira: BALINA once (saldiri)'),('prorata','orantili dagitim'),('exempt_small','kucuk satis muaf, balina sinirli'),('exempt_split','balina kucuge BOLER (muaf)')):
                a=ev(mode,cap,0.75,scen); print(f"%{int(cap*100)} {name:31} {a[0]:6.1f} | {a[1]:6.1f} | {a[2]:5.1f}")
