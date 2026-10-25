"""Denetim ajani referans modeli: sabit kurallar, gecici eylem, komite eskalasyonu. Sentetik veri."""
import random
from collections import defaultdict
LIMIT=10_000      # adres basi tek islem esigi (USD)
STRUCT_N=4        # pencerede LIMIT'in %80-100'u araliginda islem sayisi
WIN=24*3600
RAPID=3600        # girisle cikis arasi sure
def gen(seed,n_legit=3000,n_att=60):
    r=random.Random(seed); tx=[]; lab={}   # tx: (t,from,to,usd)
    sanc={f'S{i}' for i in range(20)}
    for i in range(n_legit):
        a=f'L{i}'; lab[a]='legit'
        for _ in range(r.randint(1,12)):
            tx.append((r.randint(0,7*86400),a,'POOL',min(r.lognormvariate(6,1.5),9500)))
    for i in range(n_att):
        k=r.choice(['sanc','struct','rapid','fan']); a=f'A{i}'; lab[a]=k if k in ('struct','rapid') else 'dummy'
        t0=r.randint(0,6*86400)
        if k=='sanc': sa=f'S{r.randint(0,19)}'; tx.append((t0,sa,'POOL',r.uniform(500,50000))); lab[sa]='sanc'
        if k=='struct':
            for j in range(r.randint(5,9)): tx.append((t0+j*r.randint(600,3600),a,'POOL',r.uniform(8000,9990)))
        if k=='rapid':
            tx.append((t0,'EXT',a,r.uniform(5000,40000))); tx.append((t0+r.randint(60,3000),a,'POOL',r.uniform(4000,39000)))
        if k=='fan':
            f=f'F{i}'; lab[f]='fan'
            for j in range(r.randint(8,15)):
                w=f'W{i}_{j}'; lab[w]='fan'; tx.append((t0+j*60,f,w,r.uniform(900,1500))); tx.append((t0+j*60+r.randint(100,900),w,'POOL',r.uniform(800,1400)))
    return sorted(tx),lab,sanc
def agent(tx,sanc):
    flag={}  # adres -> (kural, karar_turu)
    seen=defaultdict(list); inflow={}; fund=defaultdict(set)
    for t,a,b,u in tx:
        if a in sanc: flag.setdefault(a,('sanc','OTOMATIK')) ; continue
        if b in sanc: flag.setdefault(b,('sanc','OTOMATIK'))
        if LIMIT*.8<=u<LIMIT: 
            seen[a]=[x for x in seen[a] if t-x<WIN]+[t]
            if len(seen[a])>=STRUCT_N: flag.setdefault(a,('struct','KOMITE'))
        if a=='EXT': inflow[b]=(t,u)
        if a in inflow and b=='POOL' and t-inflow[a][0]<RAPID and u>=.7*inflow[a][1] and inflow[a][1]>=5000: flag.setdefault(a,('rapid','KOMITE'))
        if b.startswith('W') and a.startswith('F'):
            fund[a].add(b)
            if a in flag: flag.setdefault(b,('fan','KOMITE'))
        if len(fund.get(a,()))>=8: 
            for w in fund[a]: flag.setdefault(w,('fan','KOMITE')); 
            flag.setdefault(a,('fan','KOMITE'))
    return flag
def run(seeds=range(20)):
    tp=fn=fp=n_att=n_leg=auto_fp=0
    for s in seeds:
        tx,lab,sanc=gen(s); f=agent(tx,sanc)
        for a,l in lab.items():
            if l=='dummy': continue
            if l=='legit':
                n_leg+=1
                if a in f: fp+=1; auto_fp+= f[a][1]=='OTOMATIK'
            else:
                n_att+=1
                if a in f: tp+=1
                else: fn+=1
    return dict(saldiri=n_att,yakalanan=tp,kacan=fn,yakalama=round(tp/n_att,3),masum=n_leg,yanlis_alarm=fp,yanlis_oran=round(fp/n_leg,5),otomatik_yanlis=auto_fp)
def tests():
    f=[]
    tx=[(0,'S1','POOL',100)]; 
    if agent(tx,{'S1'}).get('S1',(0,0))[1]!='OTOMATIK': f.append('yaptirim kacti')
    tx=[(i*100,'X','POOL',9000) for i in range(3)]
    if 'X' in agent(tx,set()): f.append('3 islem esigi altinda tetikledi')
    tx=[(i*100,'X','POOL',9000) for i in range(4)]
    if 'X' not in agent(tx,set()): f.append('parcalama kacti')
    tx=[(i*100000,'X','POOL',9000) for i in range(6)]
    if 'X' in agent(tx,set()): f.append('seyrek islem yanlis alarm')
    tx=[(0,'EXT','Y',9000),(100,'Y','POOL',8800)]
    if 'Y' not in agent(tx,set()): f.append('hizli giris-cikis kacti')
    tx=[(0,'EXT','Y',9000),(7200,'Y','POOL',8800)]
    if 'Y' in agent(tx,set()): f.append('yavas cikis yanlis alarm')
    return f
if __name__=='__main__':
    print('birim:','HEPSI GECTI' if not tests() else tests())
    print(run())
