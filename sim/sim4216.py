import random, math, statistics as st
random.seed(7)
CE=200.0; W=104
SC={'felaket':(150,-0.01,1.0),'kotu':(300,0.00,0.9),'orta':(1500,0.02,0.8),'iyi':(6000,0.03,0.7)}  # baslangic haftalik gelir, haftalik buyume, volatilite
def run(floor,cap_w=208,ins_w=52,sc='orta',presale=20800):
    r0,g,vol=SC[sc]
    res=presale; ins=0.0; buy=stk=sec=0.0; deficit=0.0; minres=res; hit_floor=False; ruin=False; cap_week=None
    for w in range(W):
        rev=r0*((1+g)**w)*math.exp(random.gauss(0,vol)-vol*vol/2)
        net=rev-CE
        if net<0:
            take=min(res,-net); res-=take; deficit+=(-net-take)
            if res<=0: ruin=True
        else:
            capv=cap_w*CE
            to_res=min(net*floor, max(0,capv-res)); res+=to_res
            rest=net-to_res
            ti=min(rest*0.10,max(0,ins_w*CE-ins)); ins+=ti; rest-=ti
            s=rest*0.10; sec+=s; rest-=s
            buy+=rest*0.5; stk+=rest*0.5
            if res>=capv and cap_week is None: cap_week=w
        minres=min(minres,res)
        if res<ins_w*CE*0 + 26*CE and net<0: pass
        if res<26*CE: hit_floor=True
    return res,ins,buy,stk,sec,deficit,ruin,cap_week,hit_floor
N=3000
import sys
print('senaryo taban | rezerv_ort rezerv_min<26hafta% iflas% tavana_ulasma% sigorta buyback staker (USD, 2 yil)')
out=[]
for sc in SC:
    for fl in (0.15,0.25,0.40):
        rs=[run(fl,sc=sc) for _ in range(N)]
        f=lambda i: st.mean(x[i] for x in rs)
        line=f"{sc:5} %{int(fl*100):2} | rez {f(0):10.0f}  alt26h {100*sum(x[8] for x in rs)/N:5.1f}%  iflas {100*sum(x[6] for x in rs)/N:5.1f}%  tavan {100*sum(x[7] is not None for x in rs)/N:5.1f}%  sig {f(1):8.0f} buyback {f(2):10.0f} staker {f(3):10.0f}"
        print(line); out.append(line)
# balina testi
print('\nBalina testi: havuz derinligi x (token), balina satisi W*x; fiyat orani = (x/(x+s))^2; arbitraj her epoch acigin %50 kapatir')
def dump(Wf,cap):
    price=1.0; gap=0.0; left=Wf; mx=0; ep=0; 
    while left>1e-9 and ep<500:
        s=min(left,cap) if cap else left
        left-=s; ratio=(1/(1+s))**2
        price=price*ratio
        mx=max(mx,1-price)
        price=price+(1-price)*0.5  # kismi toparlanma sonraki epocha
        ep+=1
    return mx,ep
for Wf in (0.2,0.5,1.0):
    a=dump(Wf,None); b=dump(Wf,0.05)
    print(f"Satis={int(Wf*100)}% derinlik: tavansiz max dusus {a[0]*100:5.1f}% ({a[1]} epoch) | %5 tavanli max dusus {b[0]*100:5.1f}% ({b[1]} epoch)")
