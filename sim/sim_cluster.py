import random, math, statistics as st
# Birim: DK, hafta 1 (0.02 USD). Haftalik arz 462,200 DK (9,244 USD). Cuzdan/kume siniri 10,000 DK.
SUP=462200; CAP=10000; N=300; WH_BUDGET=1_000_000  # balina 20k USD ~ 1M DK
def wf(reqs,sup):
    if sum(reqs)<=sup: return list(reqs)
    lo,hi=0,max(reqs)
    for _ in range(60):
        L=(lo+hi)/2
        if sum(min(r,L) for r in reqs)>sup: hi=L
        else: lo=L
    return [min(r,lo) for r in reqs]
def run(k,e,fp,cluster,T=200):
    wsh=[];hsh=[];hbase=[]
    for t in range(T):
        r=random.Random(t)
        hon=[min(CAP,max(1000,r.lognormvariate(math.log(5000),0.8))) for _ in range(N)]
        # gruplar: her grup tek sinir. (talep listesi, sahip)
        groups=[]  # (toplam talep, ust sinir, tur, idx)
        # durustler: fp orani 2-4'lu kumeye yanlislikla girer
        idx=list(range(N)); r.shuffle(idx); i=0; hg=[]
        while i<N:
            if cluster and r.random()<fp:
                n=r.randint(2,4); hg.append(idx[i:i+n]); i+=n
            else: hg.append([idx[i]]); i+=1
        per=min(CAP,WH_BUDGET/k)
        wal=[per]*k
        hidden=[r.random()<e for _ in range(k)]   # tespit edilemeyen balina cuzdani
        det=[w for w,h in zip(wal,hidden) if not h]
        und=[w for w,h in zip(wal,hidden) if h]
        reqs=[];meta=[]
        for g in hg:
            reqs.append(sum(hon[j] for j in g)); meta.append(('h',g))
        # kume siniri: kume = tek cuzdan siniri
        capg=lambda tot,cl: min(tot,CAP) if cl else tot
        reqs=[capg(rq,cluster and len(m[1])>1) if m[0]=='h' else rq for rq,m in zip(reqs,meta)]
        if cluster and det:
            reqs.append(min(sum(det),CAP)); meta.append(('w',None))
        elif det:
            for w in det: reqs.append(w); meta.append(('w',None))
        for w in und: reqs.append(w); meta.append(('w',None))
        a=wf(reqs,SUP)
        wsh.append(sum(x for x,m in zip(a,meta) if m[0]=='w')/SUP)
        # durust toplam, kumesiz referansla kiyas
        hsh.append(sum(x for x,m in zip(a,meta) if m[0]=='h'))
        # referans: kume yok, balina yok
        b=wf(hon,SUP); hbase.append(sum(b))
    return 100*st.mean(wsh), 100*(1-st.mean(hsh)/st.mean(hbase))
print("Balinanin haftalik paydaki yuzdesi | k=balina cuzdani, e=tespitten kacan oran, fp=durust cuzdanlarin yanlis kumelenme orani")
print("k   e     | kume yok | kume (fp=0%) | kume (fp=10%)  [son sutun: durust alicilarin kaybi %]")
for k in (50,100,200):
  for e in (0,0.5,0.9,1.0):
    a,_=run(k,e,0,False); b,_=run(k,e,0,True); c,h=run(k,e,0.10,True)
    print(f"{k:3d} {e:.1f}   | %{a:5.1f}   | %{b:5.1f}       | %{c:5.1f}   (durust kayip %{h:4.1f})")
