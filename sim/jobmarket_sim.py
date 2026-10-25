"""Ajan is pazari: izinsiz giris vs komite onayi vs komite+deneme suresi.
PARAMETRELER VARSAYIMDIR (gercek veri degil); amac mekanizmalarin davranisini karsilastirmak."""
import random, math, statistics as st
def poisson(r,l):
    L=math.exp(-l); k=0; p=1.0
    while p>L: k+=1; p*=r.random()
    return k-1
def run(policy,seed,days=60,rate=5,mal=0.3,colluding=0,committee=5,need=3,detect=0.7,false_rej=0.10,
        review_cap=20,verify=0.9,bond=50,pay=10,tasks=2,prob_days=7,prob_frac=0.2):
    r=random.Random(seed); queue=[]; active=[]
    m=dict(honest_acc=0,honest_rej=0,mal_acc=0,mal_rej=0,delay=[],damage=0.0,value=0.0,slashed=0.0,reviews=0,backlog_max=0)
    for d in range(days):
        for _ in range(poisson(r,rate)): queue.append(dict(t=d,mal=r.random()<mal,start=None))
        if policy=='open':
            for a in queue: a['start']=d; active.append(a); m['mal_acc' if a['mal'] else 'honest_acc']+=1; m['delay'].append(0)
            queue=[]
        else:
            cap=review_cap; keep=[]
            for a in queue:
                if cap<=0 or d<a['t']+1: keep.append(a); continue
                cap-=1; m['reviews']+=1; votes=0
                for k in range(committee):
                    if k<colluding: votes+= 1 if a['mal'] else (r.random()>false_rej)
                    elif a['mal']: votes+= (r.random()>detect)
                    else: votes+= (r.random()>false_rej)
                if votes>=need:
                    a['start']=d; active.append(a); m['mal_acc' if a['mal'] else 'honest_acc']+=1; m['delay'].append(d-a['t'])
                else: m['mal_rej' if a['mal'] else 'honest_rej']+=1
            queue=keep; m['backlog_max']=max(m['backlog_max'],len(queue))
        for a in list(active):
            for _ in range(tasks):
                p=pay*(prob_frac if policy=='committee_prob' and d-a['start']<prob_days else 1)
                if a['mal']:
                    if r.random()<verify: m['slashed']+=bond; active.remove(a); break
                    m['damage']+=p
                else: m['value']+=p
    return m
def agg(policy,**kw):
    S=[run(policy,s,**kw) for s in range(200)]
    f=lambda k: st.mean(x[k] for x in S)
    ma=f('mal_acc'); ha=f('honest_acc'); mr=f('mal_rej'); hr=f('honest_rej')
    return dict(zararli_kabul=round(ma/max(ma+mr,1e-9),3),durust_ret=round(hr/max(ha+hr,1e-9),3),
        gecikme_gun=round(st.mean([d for x in S for d in x['delay']] or [0]),2),zarar=round(f('damage')),
        kesilen_teminat=round(f('slashed')),durust_deger=round(f('value')),birikim=round(f('backlog_max'),1))
def tests():
    f=[]
    a=run('open',1,mal=0.0); 
    if a['damage']!=0 or a['mal_acc']!=0: f.append('zararlisiz ortamda zarar')
    b=run('committee',1,mal=1.0,detect=1.0,colluding=0)
    if b['mal_acc']!=0: f.append('kusursuz komite zararliyi kabul etti')
    c=run('committee',1,mal=1.0,colluding=5)
    if c['mal_rej']!=0: f.append('tamamen ele gecirilmis komite zararliyi reddetti')
    d=run('committee',1,mal=0.0,false_rej=0.0,review_cap=0)
    if d['honest_acc']!=0: f.append('kapasite 0 iken kabul')
    return f
if __name__=='__main__':
    print('birim:','HEPSI GECTI' if not tests() else tests())
    for pol in ('open','committee','committee_prob'):
        print(pol,'|',agg(pol))
    print('--- komite uyeleri ele gecirilirse (committee_prob) ---')
    for k in (0,1,2,3,4):
        print('ele gecirilen uye',k,'|',agg('committee_prob',colluding=k))
    print('--- yogun basvuru (rate=40, kapasite 20) ---')
    for pol in ('open','committee','committee_prob'): print(pol,'|',agg(pol,rate=40))
