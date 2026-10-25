"""Sosyal Medya ajani referans motoru: yalniz sablon, slot dogrulama, zorunlu risk satiri, yasakli ifade filtresi, limit."""
import re, unicodedata, random
RISK='Experimental project. Unaudited. No returns promised. You can lose everything. Official address: {REGISTRY}'
T={
-10:'DKEEY: wallet-based, identity-free, admin-minimized spot exchange, run by AI agents. Open code: {REPO}. Presale starts {START}. {RISK}',
-9:'Orders match in batch auctions at one clearing price. No admin keys; core upgrade authority closes for good after the audit. {RISK}',
-8:'All simulations and test vectors are open: {REPO}. Run them yourself and compare. {RISK}',
-7:'Presale funds stay locked in the contract. Below the minimum ({MIN} USD) everyone withdraws in full, no deadline. {RISK}',
-6:'Known limits: no independent audit, agent robustness and on-chain signature cost not measured. This can lose money. {RISK}',
-5:'Presale: 3 weeks. Prices {P1} / {P2} / {P3} USD. Cap {CAP}, minimum {MIN} USD. Excess is refunded. Paid in SOL; USD values move with the rate. {RISK}',
-4:'FAQ: money is in the contract; rules are fixed in code; not audited yet; no returns promised. {RISK}',
-3:'Our model stays alive 26 weeks if the minimum is reached. A model, not proof; demand is an assumption. {RISK}',
-2:'Scam warning: we never DM and never ask for keys or seed phrases. Only trust {REGISTRY}. {RISK}',
-1:'Presale starts tomorrow. Official: {REPO}. Compare any address with {REGISTRY} before sending. {RISK}',
}
SLOT={'REPO':r'github\.com/dkeeyDK/dkeey','REGISTRY':r'[1-9A-HJ-NP-Za-km-z]{32,44}','START':r'\d{4}-\d{2}-\d{2} \d{2}:\d{2} UTC',
 'MIN':r'\d{1,3}(,\d{3})*','CAP':r'\d{1,3}(,\d{3})*','P1':r'0\.\d{2,5}','P2':r'0\.\d{2,5}','P3':r'0\.\d{2,5}'}
BAN=[r'\d+\s*x\b',r'moon',r'guarant',r'risk[\s-]*free',r'100\s*%',r'get\s*rich',r'pump',r'airdrop',r'send\s+(funds|btc|sol|usdt)',r'dm\s+me',r'seed\s*phrase\s*:',r'private\s*key\s*:',r'profit',r'passive\s+income',r'will\s+(rise|increase|go\s+up)']
LIMIT={'bluesky':300,'nostr':2000}
PER_DAY={'bluesky':3,'nostr':3,'github':1}
HOMO=str.maketrans({'а':'a','е':'e','о':'o','р':'p','с':'c','х':'x','у':'y','і':'i','ѕ':'s'})
def norm(s): return unicodedata.normalize('NFKC',s).translate(HOMO).lower()
class Reject(Exception): pass
def render(day,slots,platform,posted_today=0,registered_domain=False):
    if day not in T: raise Reject('bilinmeyen sablon')
    if platform not in PER_DAY: raise Reject('bilinmeyen platform')
    if posted_today>=PER_DAY[platform]: raise Reject('gunluk limit')
    t=T[day].replace('{RISK}',RISK); need=set(re.findall(r'\{(\w+)\}',t))
    if set(slots)-need: raise Reject('fazla slot')
    if need-set(slots): raise Reject('eksik slot')
    out=t
    for k in need:
        v=slots[k]
        if not re.fullmatch(SLOT[k],v): raise Reject('slot gecersiz '+k)
        out=out.replace('{'+k+'}',v)
    n=norm(out)
    for b in BAN:
        if re.search(b,n): raise Reject('yasakli ifade')
    if 'dkeey.com' in n and not registered_domain: raise Reject('alan adi kayitli degil')
    for u in re.findall(r'(?:https?://)?([a-z0-9.-]+\.[a-z]{2,})(?:/\S*)?',n):
        if u not in ('github.com',) and not (u=='dkeey.com' and registered_domain): raise Reject('izinsiz baglanti '+u)
    if len(out)>LIMIT.get(platform,10**6): raise Reject('uzunluk %d'%len(out))
    if 'official address' not in n and 'official:' not in n and 'only trust' not in n: raise Reject('risk/resmi adres yok')
    return out
OK=dict(REPO='github.com/dkeeyDK/dkeey',REGISTRY='npub1h5zr7wvg8npzw3uscnjkkyg8r7cyfp2g9qhztxd59qch5c92fcusqr3kpa'[:44],START='2026-12-01 12:00 UTC',MIN='13,867',CAP='27,733',P1='0.02',P2='0.025',P3='0.03125')
def tests():
    f=[]
    for d,t in T.items():
        need=set(re.findall(r'\{(\w+)\}',t.replace('{RISK}',RISK)))
        try: out=render(d,{k:OK[k] for k in need},'bluesky')
        except Reject as e: f.append(f'gun {d}: {e}')
    def must_reject(name,day,slots,plat='bluesky',**kw):
        try: render(day,slots,plat,**kw); f.append('kabul edildi: '+name)
        except Reject: pass
    base=lambda d:{k:OK[k] for k in set(re.findall(r'\{(\w+)\}',T[d].replace('{RISK}',RISK)))}
    s=base(-10); s['REPO']='github.com/evil/phish'; must_reject('sahte repo',-10,s)
    s=base(-2); s['REGISTRY']='short'; must_reject('kisa adres',-2,s)
    s=base(-10); must_reject('fazla slot',-10,dict(s,X='1'))
    must_reject('eksik slot',-10,{'REPO':OK['REPO']})
    must_reject('limit',-10,base(-10),posted_today=3)
    must_reject('platform',-10,base(-10),'x')
    s=base(-5); s['P1']='0.02 get rich'; must_reject('slotta ifade',-5,s)
    s=base(-10); s['START']='2026-12-01 12:00 UTC'; 
    for evil in ['guaranteed','100x','m0on'.replace('0','o'),'Ｇuarantee','100 x','risk free','risk-free']:
        TT=dict(T); T[-10]=T[-10]+' '+evil
        try: render(-10,s,'nostr'); f.append('yasakli gecti: '+evil)
        except Reject: pass
        T.clear(); T.update(TT)
    TT=dict(T); T[-10]=T[-10]+' dkeey.com'
    try: render(-10,s,'nostr'); f.append('kayitsiz alan adi gecti')
    except Reject: pass
    try: render(-10,s,'nostr',registered_domain=True)
    except Reject as e: f.append('kayitli alan adi reddedildi: '+str(e))
    T.clear(); T.update(TT)
    return f
def fuzz(n=100000,seed=5):
    r=random.Random(seed); bad=0; ok=0
    al='abcdefghijklmnopqrstuvwxyz0123456789 .:/-_,x%'
    for _ in range(n):
        d=r.choice(list(T)); need=set(re.findall(r'\{(\w+)\}',T[d].replace('{RISK}',RISK)))
        sl={k:(OK[k] if r.random()<.6 else ''.join(r.choice(al) for _ in range(r.randint(0,30)))) for k in need}
        try:
            out=render(d,sl,r.choice(['bluesky','nostr']))
            ok+=1; low=norm(out)
            # kabul edilen metinde yasakli ifade / izinsiz baglanti olamaz
            if any(re.search(b,low) for b in BAN) or 'official' not in low and 'only trust' not in low: bad+=1
        except Reject: pass
    return n,ok,bad
if __name__=='__main__':
    print('birim:', 'HEPSI GECTI' if not tests() else tests())
    print('fuzz (deneme, kabul, ihlal):',fuzz())
    print('uzunluklar:',{d:len(T[d].replace('{RISK}',RISK))+0 for d in T})
