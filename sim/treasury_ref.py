"""Treasury modulu referans modeli: epoch basina cekim tavani (Normal %0,5, Crisis %1), sabit hedef (ops), yalniz yetkili imza."""
import random
NORMAL_BPS=50; CRISIS_BPS=100
class Err(Exception): pass
class Treasury:
    def __init__(s,start,epoch_secs,authority,balance=0):
        s.start=start; s.es=epoch_secs; s.auth=authority; s.bal=balance; s.ops=0
        s.epoch=-1; s.snap=0; s.wd=0; s.crisis=False; s.total_in=balance; s.total_out=0
    def deposit(s,x):
        if x<=0: raise Err('miktar')
        s.bal+=x; s.total_in+=x
    def set_mode(s,caller,crisis):
        if caller!=s.auth: raise Err('yetki')
        s.crisis=crisis
    def withdraw(s,caller,now,x):
        if caller!=s.auth: raise Err('yetki')
        if x<=0 or now<s.start: raise Err('miktar')
        e=(now-s.start)//s.es
        if e<s.epoch: raise Err('zaman geri')
        if e!=s.epoch: s.epoch=e; s.snap=s.bal; s.wd=0
        cap=s.snap*(CRISIS_BPS if s.crisis else NORMAL_BPS)//10000
        if s.wd+x>cap: raise Err('tavan')
        if x>s.bal: raise Err('bakiye')
        s.bal-=x; s.ops+=x; s.wd+=x; s.total_out+=x
def check(t):
    assert t.bal>=0 and t.ops>=0
    assert t.bal+t.ops==t.total_in
    assert t.wd<=t.snap*CRISIS_BPS//10000
def tests():
    A,B=1,2; es=100; f=[]
    def rej(name,fn):
        try: fn(); f.append('kabul edildi: '+name)
        except Err: pass
    t=Treasury(0,es,A,1_000_000)
    t.withdraw(A,10,5000); assert t.ops==5000          # %0,5 = 5000
    rej('tavan asimi',lambda:t.withdraw(A,20,1))        # ayni epoch, tavan dolu
    rej('yetkisiz',lambda:t.withdraw(B,120,1))
    t.withdraw(A,120,4975); assert t.ops==9975          # yeni epoch: snap 995000 -> tavan 4975
    rej('geri zaman',lambda:t.withdraw(A,50,1))
    t2=Treasury(0,es,A,1_000_000)
    rej('bakiye',lambda:t2.withdraw(A,10,2_000_000))
    t2.withdraw(A,10,5000); t2.set_mode(A,True); t2.withdraw(A,20,5000); assert t2.ops==10000  # Crisis tavan %1=10000
    rej('crisis tavani',lambda:t2.withdraw(A,30,1))
    rej('yetkisiz mod',lambda:t2.set_mode(B,False))
    # bos epoch birikmez
    t3=Treasury(0,es,A,1_000_000)
    rej('birikim yok',lambda:t3.withdraw(A,10_000,5001))
    for x in (t,t2,t3): check(x)
    return f
def fuzz(n=20000,seed=3):
    r=random.Random(seed); viol=0; wd_total=0
    for _ in range(n):
        es=r.randint(10,1000); bal=r.randint(0,10**9)
        t=Treasury(0,es,1,bal); now=0; init=bal
        epochs_used={}
        for _ in range(r.randint(1,60)):
            now+=r.randint(0,3*es); who=1 if r.random()<.9 else 2
            if r.random()<.2: t.deposit(r.randint(1,10**7))
            if r.random()<.1:
                try: t.set_mode(who,r.random()<.5)
                except Err: pass
            x=r.randint(1,max(1,t.bal//50+1))
            try: t.withdraw(who,now,x)
            except Err: pass
            check(t)
        # her epoch icin cikis <= %1 o epoch baslangic bakiyesi: check() zaten dogrular
        wd_total+=t.ops
    return n,viol
if __name__=='__main__':
    print('birim:','HEPSI GECTI' if not tests() else tests())
    print('fuzz (kosu, ihlal):',fuzz())
