"""Sandvic saldirisi karsilastirmasi: sirali AMM (acik mempool) vs toplu muzayede (tek fiyat).
Stilize model: gercek piyasa verisi DEGIL, mekanizma karsilastirmasi. Fiyat birimi: quote/token, baslangic fiyat 1."""
import random, statistics as st, sys
sys.path.insert(0,'/home/claude/sim')
FEE=0.003   # havuz ucreti
def amm_buy(x,y,q):      # x=quote, y=token, q=quote gonderilir -> token alinir
    qf=q*(1-FEE); out=y-x*y/(x+qf); return out,x+q,y-out
def amm_sell(x,y,t):     # t token satilir -> quote alinir
    tf=t*(1-FEE); out=x-x*y/(y+tf); return out,x-out,y+t
def sandwich(depth,Q,tol):
    """kurban Q quote ile alim yapar, min cikti = (1-tol)*beklenen. Saldirgan en buyuk karli on-alimi arar."""
    x=y=depth
    exp,_,_=amm_buy(x,y,Q); base=exp; minout=exp*(1-tol)
    lo,hi=0.0,depth*5; best=(0.0,0.0,base)
    for _ in range(60):
        a=(lo+hi)/2
        t,x1,y1=amm_buy(x,y,a)
        v,x2,y2=amm_buy(x1,y1,Q)
        if v>=minout: lo=a
        else: hi=a
    a=lo
    if a<=0: return 0.0,0.0,base
    t,x1,y1=amm_buy(x,y,a); v,x2,y2=amm_buy(x1,y1,Q); s,_,_=amm_sell(x2,y2,t)
    prof=s-a
    if prof<=0: return 0.0,0.0,base
    return prof,(base-v)/base,v     # kar, kurban kaybi orani
def amm_grid(seed=1,n=4000):
    r=random.Random(seed); res=[]
    for _ in range(n):
        depth=10**r.uniform(4.5,7.5); Q=depth*10**r.uniform(-3.5,-1.3); tol=r.choice([0.005,0.01,0.03,0.05])
        prof,loss,_=sandwich(depth,Q,tol)
        res.append((prof>0,loss,prof/Q,tol))
    return res
# ---- toplu muzayede ----
def clear(orders):
    os_=sorted(orders,key=lambda o:o[0]); prices=sorted({o[2] for o in os_}); best=None
    for p in prices:
        D=sum(o[3] for o in os_ if o[1]=="B" and o[2]>=p); S=sum(o[3] for o in os_ if o[1]=="S" and o[2]<=p)
        key=(min(D,S),-abs(D-S),p)
        if best is None or key>best[0]: best=(key,p)
    if best is None or best[0][0]==0: return None,0,{}
    p=best[1]; V=best[0][0]
    buys=sorted([o for o in os_ if o[1]=="B" and o[2]>=p],key=lambda o:(-o[2],o[0]))
    sells=sorted([o for o in os_ if o[1]=="S" and o[2]<=p],key=lambda o:(o[2],o[0]))
    fills={}
    for side in (buys,sells):
        rem=V
        for o in side:
            f=min(o[3],rem); rem-=f
            if f: fills[o[0]]=f
    return p,V,fills
def batch_trial(r,att_frac):
    bg=[(f"b{i}",r.choice("BS"),round(r.uniform(97,103),1),r.randint(5,60)) for i in range(r.randint(30,80))]
    vq=r.randint(20,200)
    victim=("victim","B",110.0,vq)
    p0,V0,f0=clear(bg+[victim])
    if p0 is None or "victim" not in f0: return None
    a=max(1,int(vq*att_frac))
    atk=[("atkB","B",110.0,a),("atkS","S",90.0,a)]
    p1,V1,f1=clear(bg+[victim]+atk)
    if p1 is None or "victim" not in f1: return None
    fb=f1.get("atkB",0); fs=f1.get("atkS",0)
    pnl=(fs-fb)*p1          # ayni fiyattan: kar yok, sadece envanter farki
    fee=(fs+fb)*p1*0.001
    return (p1-p0)/p0, pnl-fee, fb, fs, f1["victim"]/vq
def batch_grid(seed=2,n=4000):
    r=random.Random(seed); out=[]
    for _ in range(n):
        t=batch_trial(r,r.choice([0.5,1,3,10,30]))
        if t: out.append(t)
    return out
def batch_manip(r,att_frac):
    """Daha guclu saldiri: tek tarafli buyuk alis emriyle fiyati yukari itmek. Kurban zarari vs saldirgan maliyeti."""
    bg=[(f"b{i}",r.choice("BS"),round(r.uniform(97,103),1),r.randint(5,60)) for i in range(r.randint(30,80))]
    vq=r.randint(20,200); victim=("victim","B",110.0,vq)
    p0,V0,f0=clear(bg+[victim])
    if p0 is None or "victim" not in f0: return None
    a=max(1,int(vq*att_frac))
    p1,V1,f1=clear(bg+[victim,("atk","B",110.0,a)])
    if p1 is None or "victim" not in f1 or "atk" not in f1: return None
    vloss=f1["victim"]*(p1-p0)+ (f0["victim"]-f1["victim"])*0   # kurbanin ayni miktar icin ekstra odemesi
    acost=f1["atk"]*(p1-p0)  # saldirganin adil fiyata gore ekstra odemesi (bunu bir yerden telafi etmesi gerek)
    return (p1-p0)/p0, vloss, acost
def manip_grid(seed=3,n=4000):
    r=random.Random(seed); out=[]
    for _ in range(n):
        t=batch_manip(r,r.choice([1,3,10,30,100]))
        if t: out.append(t)
    return out
def tests():
    f=[]
    # AMM: saldirisiz alim, kurban kaybi sifir
    p,l,_=sandwich(1e6,1,0.0005)
    if l>0.0007: f.append('cok dusuk tolerans yine de buyuk kayip')
    # tolerans arttikca kayip artar
    a=sandwich(1e6,20000,0.01)[1]; b=sandwich(1e6,20000,0.05)[1]
    if not b>=a: f.append('kayip toleransla artmali')
    # kayip toleransi asamaz
    for tol in (0.005,0.03):
        if sandwich(1e6,30000,tol)[1]>tol+1e-6: f.append('kayip toleransi asti')
    # toplu: saldirgan tek fiyattan alir/satar, kar sifir (envanter farki disinda)
    r=random.Random(0)
    for _ in range(300):
        t=batch_trial(r,3)
        if t and abs(t[1])>t[2]*1000+t[3]*1000: f.append('saldirgan kari anormal'); break
    return f
if __name__=='__main__':
    print('birim:','HEPSI GECTI' if not tests() else tests())
    A=amm_grid(); n=len(A)
    pr=[a for a in A if a[0]]
    print(f"AMM (n={n}): saldirinin karli oldugu islem orani {len(pr)/n:.1%}")
    print(f"  karli saldirilarda kurban kaybi: ortalama {st.mean(a[1] for a in pr):.2%}, medyan {st.median(a[1] for a in pr):.2%}, maks {max(a[1] for a in pr):.2%}")
    for tol in (0.005,0.01,0.03,0.05):
        s=[a for a in A if a[3]==tol]; ps=[a for a in s if a[0]]
        print(f"  tolerans {tol:.1%}: karli {len(ps)/len(s):.0%}, tum islemlerde ortalama kayip {st.mean(a[1] for a in s):.2%}")
    B=batch_grid(); m=len(B)
    sh=[abs(b[0]) for b in B]; pnl=[b[1] for b in B]
    print(f"Toplu muzayede (n={m}): saldirgan eklenince fiyat kaymasi ortalama {st.mean(sh):.3%}, maks {max(sh):.3%}")
    M=manip_grid(); k=len(M)
    up=[x for x in M if x[1]>0]
    print(f"Toplu muzayede, tek tarafli fiyat itme (n={k}): fiyat kaymasi olan {len(up)/k:.1%}; ortalama kayma {st.mean(x[0] for x in M):.3%}")
    if up: print(f"  kayma olunca: kurban ekstra odeme ort {st.mean(x[1] for x in up):.2f}, saldirgan ekstra maliyet ort {st.mean(x[2] for x in up):.2f}, oran saldirgan/kurban {st.mean(x[2] for x in up)/st.mean(x[1] for x in up):.2f}")
    print(f"  saldirgan net kari: ortalama {st.mean(pnl):.2f}, pozitif orani {sum(p>0 for p in pnl)/m:.1%} (birim: emir-birimi x fiyat)")
