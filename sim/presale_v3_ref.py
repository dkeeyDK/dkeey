"""presale_program_v3.rs on-satis mantiginin Python referansi (ayni kova algoritmasi).
Kontroller: para korunumu, cuzdan basina 10K DK, balina payi, rollover. Modeldir, kanit degildir."""
import random
B=128; MAXDK=10_000
def bound(mx,k): return mx*(k+1)*(k+1)//16384
def bucket_of(mx,r):
    for k in range(B):
        if r<=bound(mx,k): return k
def fair_level(room,mx,cnt,sm,mn):
    tot=sum(sm)
    if tot<=room: return (2**64-1,tot)
    nt=sum(cnt); below=nb=0; best=(0,0,-1)
    for k in range(B):
        below+=sm[k]; nb+=cnt[k]; l=bound(mx,k); f=below+l*(nt-nb)
        if f<=room: best=(l,f,k)
        else: break
    l,f,kb=best
    na=sum(c for j,c in enumerate(cnt) if j>kb)
    if na==0: return (l,f)
    ma=min([mn[j] for j in range(B) if j>kb and cnt[j]>0] or [l])
    ex=min((room-f)//na, max(0,ma-l))
    return (l+ex, f+ex*na)

class Sale:
    def __init__(s,cap,minr,rates):
        s.cap,s.min,s.rates=cap,minr,rates; s.raised=0
        s.cnt=[[0]*B for _ in range(3)]; s.sm=[[0]*B for _ in range(3)]; s.mn=[[2**64]*B for _ in range(3)]
        s.level=[2**64-1]*3; s.req={}; s.escrow=0
    def contribute(s,w,who,amt):
        r=s.req.setdefault(who,[0,0,0])
        used=sum(r[i]*10**12//s.rates[i] for i in range(3)); capq=MAXDK*10**12
        if used>=capq: return 0
        take=min(amt,(capq-used)*s.rates[w]//10**12)
        mx=MAXDK*s.rates[w]; old=r[w]; new=old+take
        if take<=0 or new>mx: return 0
        if old>0:
            b=bucket_of(mx,old); s.cnt[w][b]-=1; s.sm[w][b]-=old
        b=bucket_of(mx,new); s.cnt[w][b]+=1; s.sm[w][b]+=new; s.mn[w][b]=min(s.mn[w][b],new)
        r[w]=new; s.escrow+=take; return take
    def settle(s,w):
        cum=(w+1)*s.cap//3; room=min(max(cum-s.raised,0),s.cap-s.raised)
        mx=MAXDK*s.rates[w]
        lv,al=fair_level(room,mx,s.cnt[w],s.sm[w],s.mn[w]); s.level[w]=lv; s.raised+=al; return al
    def alloc(s,who): return [min(x,s.level[i]) for i,x in enumerate(s.req[who])]
    def tokens(s,who): return sum(a//s.rates[i] for i,a in enumerate(s.alloc(who)))

def run(n_small,whale_dk=10_000,small_dk=50,cap=27_733*10**6,price=(20_000,25_000,31_250)):
    s=Sale(cap,cap//2,list(price)); rnd=random.Random(1)
    s.contribute(0,'whale',whale_dk*price[0])
    for i in range(n_small): s.contribute(0,i,small_dk*price[0])
    for w in range(3): s.settle(w)
    tot_req=sum(sum(v) for v in s.req.values())
    paid=sum(sum(s.alloc(k)) for k in s.req)
    refunds=sum(sum(s.req[k])-sum(s.alloc(k)) for k in s.req)
    assert paid==s.raised and paid+refunds==tot_req, "para korunumu bozuk"
    assert all(s.tokens(k)<=MAXDK for k in s.req)
    wt=s.tokens('whale'); tt=sum(s.tokens(k) for k in s.req)
    return wt,tt,(s.tokens(0) if n_small else 0)

if __name__=="__main__":
    print("balina: 10K DK istek; kucukler: 50 DK istek; hafta-1 odasi = cap/3")
    for n in (0,1,50,200,1000,5000):
        wt,tt,sm=run(n)
        print(f"kucuk={n:5d}  balina={wt:6d} DK  toplam={tt:8d} DK  balina payi={100*wt/max(tt,1):5.1f}%  kucuk basina={sm} DK")
    print("\nTalep > oda (hafta-1 odasi = 5.000 DK): balina 10K DK istiyor")
    for n in (1,10,50,100,500):
        wt,tt,sm=run(n,cap=3*5_000*20_000)
        print(f"kucuk={n:4d}  balina={wt:5d} DK  kucuk basina={sm:4d} DK  toplam(3 hafta)={tt:6d} DK  balina payi={100*wt/max(tt,1):5.1f}%")
