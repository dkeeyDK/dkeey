"""On satis referans modeli (26.3/26.4). Tutarlar sent (USD*100), tamsayi. Admin yok, her sey kural."""
import random
CE=200
CAP=104*CE*100*100//75           # 2.773.333 sent
MINT=52*CE*100*100//75           # 1.386.666 sent
WEEK=7*86400
PRICES=[(2000,100000),(2500,100000),(3125,100000)]  # fiyat = n/100000 USD -> 0.02,0.025,0.03125
RES_SHARE_NUM,RES_SHARE_DEN=75,100

class Presale:
    def __init__(s,t0=0):
        s.t0=t0; s.raised=0; s.contrib={}; s.tokens={}; s.state='open'
        s.refunded={}; s.paid_out=0; s.reserve=0; s.free=0; s.claimed_tokens={}; s.fin_t=None; s.released=False; s.vault=0
    def week(s,t): return min(2,(t-s.t0)//WEEK)
    def end(s): return s.t0+3*WEEK
    def room(s,t):
        w=s.week(t); return max(0,(w+1)*CAP//3 - s.raised) if w<3 else 0
    def contribute(s,u,amt,t):
        """Kismen dolum: tavan odasi asilan kisim ayni islemde iade edilir. (Not: dokumandaki 'oransal iade' sonraki surum.)"""
        if s.state!='open' or t<s.t0 or t>=s.end() or amt<=0: return (0,amt)
        take=min(amt,s.room(t)); back=amt-take
        if take==0: return (0,amt)
        n,d=PRICES[s.week(t)]
        s.contrib[u]=s.contrib.get(u,0)+take; s.raised+=take; s.vault+=take
        s.tokens[u]=s.tokens.get(u,0)+take*d//n    # sent*100000/fiyat_payi -> token birimi (ham)
        if s.raised>=CAP: s.state='capped'
        return (take,back)
    def finalize(s,t):
        if s.state in ('success','failed'): return s.state
        if s.state=='capped' or t>=s.end():
            if s.raised>=MINT:
                s.state='success'
            else: s.state='failed'
            s.fin_t=t
        return s.state
    def release(s):
        """Izinsiz, yalniz basari sonrasi, bir kez: %75 rezerv, %25 likidite ve tek seferlik gider hesabi."""
        if s.state!='success' or s.released: return None
        s.reserve=s.raised*RES_SHARE_NUM//RES_SHARE_DEN; s.free=s.raised-s.reserve
        s.vault-=s.reserve+s.free; s.released=True; return (s.reserve,s.free)
    def refund(s,u):
        if s.state!='failed' or u in s.refunded or u not in s.contrib: return 0
        a=s.contrib[u]; s.refunded[u]=a; s.paid_out+=a; s.vault-=a; return a
    def claim_tokens(s,u,t):
        """Dogrusal acilim: finalize anindan 8 hafta boyunca. Yalniz basari."""
        if s.state!='success' or u not in s.tokens: return 0
        span=8*WEEK; el=max(0,min(t-s.fin_t,span))
        vested=s.tokens[u]*el//span; x=vested-s.claimed_tokens.get(u,0)
        if x<=0: return 0
        s.claimed_tokens[u]=s.claimed_tokens.get(u,0)+x; return x
    # degismezler
    def check(s):
        e=[]
        if s.raised>CAP: e.append('tavan asildi')
        if s.raised!=sum(s.contrib.values()): e.append('toplam tutmuyor')
        if s.paid_out>s.raised: e.append('cikis>giris')
        if s.released and s.reserve+s.free!=s.raised: e.append('rezerv+serbest!=toplanan')
        if s.released and s.vault!=0: e.append('release sonrasi kasada para kaldi')
        if not s.released and s.state=='success' and s.vault!=s.raised: e.append('release oncesi kasa!=toplanan')
        if s.state=='failed' and s.vault!=s.raised-s.paid_out: e.append('basarisizda kasa tutmuyor')
        for u,c in s.claimed_tokens.items():
            if c>s.tokens[u]: e.append('claim>hak')
        if s.state=='success' and s.paid_out!=0: e.append('basarida iade')
        if s.state=='failed' and s.paid_out>s.raised: e.append('iade fazla')
        for u,a in s.refunded.items():
            if a!=s.contrib[u]: e.append('iade tam degil')
        if s.state=='failed' and s.raised>=MINT: e.append('esik asildi ama basarisiz')
        if s.state=='success' and s.raised<MINT: e.append('esik altinda basari')
        return e

def run_tests():
    P=Presale(); T=0; fails=[]
    def chk(c,m):
        if not c: fails.append(m)
    # 1 esik altinda: tam iade
    p=Presale(); p.contribute('a',500000,10); p.contribute('b',300000,WEEK+5)
    chk(p.finalize(p.end())=='failed','T1 durum')
    chk(p.refund('a')==500000 and p.refund('b')==300000,'T1 tam iade'); chk(p.refund('a')==0,'T1 cift iade'); chk(p.claim_tokens('a',p.end()+9*WEEK)==0,'T1 token yok'); chk(p.release() is None,'T1 release yok'); chk(not p.check(),'T1 degismez')
    # 2 esik uzeri: basari, rezerv kilidi
    p=Presale(); p.contribute('a',MINT,WEEK+10); chk(p.finalize(p.end())=='success','T2 durum')
    chk(p.release()==(p.raised*75//100,p.raised-p.raised*75//100),'T2 release'); chk(p.release() is None,'T2 cift release'); chk(p.vault==0,'T2 kasa bos')
    chk(p.refund('a')==0,'T2 iade yok'); t0=p.end(); chk(p.claim_tokens('a',t0)==0,'T2 claim aninda 0')
    half=p.claim_tokens('a',t0+4*WEEK); chk(half==p.tokens['a']//2,'T2 yari acilim'); chk(p.claim_tokens('a',t0+4*WEEK)==0,'T2 cift claim 0')
    rest=p.claim_tokens('a',t0+20*WEEK); chk(half+rest==p.tokens['a'],'T2 tam acilim'); chk(not p.check(),'T2 degismez')
    # 3 hafta tavani: ilk hafta CAP/3 uzeri reddedilir/iade
    p=Presale(); got,back=p.contribute('a',CAP,10); chk(got==CAP//3 and back==CAP-CAP//3,'T3 hafta tavani')
    # 4 devir: 1. hafta dolmayan 2. haftaya aktarilir
    p=Presale(); p.contribute('a',100000,10); got,back=p.contribute('b',CAP,WEEK+1); chk(got==2*CAP//3-100000,'T4 devir')
    # 5 tavan dolunca erken biter
    p=Presale(); p.contribute('a',CAP//3,1); p.contribute('a',CAP//3,WEEK+1); p.contribute('a',CAP,2*WEEK+1)
    chk(p.state=='capped' and p.raised<=CAP,'T5 erken bitis'); chk(p.finalize(2*WEEK+2)=='success','T5 basari'); chk(p.release() is not None,'T5 release')
    # 6 sure bitti: katki reddedilir
    p=Presale(); got,back=p.contribute('a',1000,p.end()+1); chk(got==0,'T6 sure sonrasi')
    # 7 fiyat kademesi: ayni tutar son haftada daha az token
    p=Presale(); p.contribute('a',100000,1); p.contribute('b',100000,2*WEEK+1); chk(p.tokens['a']>p.tokens['b'],'T7 fiyat artisi')
    # 8 basarisiz ise sonradan sonsuza dek iade (zaman siniri yok)
    p=Presale(); p.contribute('a',1000,1); p.finalize(p.end()); chk(p.refund('a')==1000,'T8 gec iade')
    return fails
def fuzz(n=20000,seed=3):
    rnd=random.Random(seed); bad=0; succ=0; fail=0
    for _ in range(n):
        p=Presale(); users=[f'u{i}' for i in range(rnd.randint(1,25))]; t=0
        for _ in range(rnd.randint(1,60)):
            t+=rnd.randint(0,WEEK//2); p.contribute(rnd.choice(users),rnd.randint(1,CAP//rnd.randint(2,40)),t)
            if p.state=='capped' and rnd.random()<.3: break
        ft=max(t,p.end()); p.finalize(ft)
        if rnd.random()<.8: p.release(); p.release()
        for u in users:
            p.refund(u); p.refund(u); p.claim_tokens(u,ft+rnd.randint(0,10*WEEK))
        e=p.check()
        if e: bad+=1
        succ+=p.state=='success'; fail+=p.state=='failed'
    return n,bad,succ,fail
if __name__=='__main__':
    print('CAP',CAP/100,'MIN',MINT/100)
    f=run_tests(); print('birim testler:', 'HEPSI GECTI' if not f else f)
    print('fuzz (kosu, ihlal, basari, basarisiz):',fuzz())
