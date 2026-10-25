"""Tamamen ajan komitesinde ortak zayiflik: ayni model ailesi vs farkli aileler. Varsayim parametreleri."""
import random
def p_capture(families, need=3, p_family=0.3, p_indiv=0.05, n=200000, seed=1):
    """families: her uye icin aile indeksi. Bir saldiri her aileyi bagimsiz p_family olasiligiyla kandirir
    (ayni ailedeki tum uyeler birlikte). Ailesi kandirilmamis uye ayrica p_indiv olasiligiyla kendi basina kanar."""
    r=random.Random(seed); fam=sorted(set(families)); ok=0
    for _ in range(n):
        fooled={f: r.random()<p_family for f in fam}
        votes=sum(1 for f in families if fooled[f] or r.random()<p_indiv)
        ok+= votes>=need
    return ok/n
def tests():
    f=[]
    if p_capture([0]*5,p_family=0,p_indiv=0)!=0: f.append('sifir saldiri etkisi')
    if p_capture([0]*5,p_family=1,p_indiv=0)!=1: f.append('tam etki')
    a=p_capture([0,0,0,0,0]); b=p_capture([0,1,2,3,4])
    if not a>b: f.append('cesitlilik yardim etmeli')
    return f
if __name__=='__main__':
    print('birim:','HEPSI GECTI' if not tests() else tests())
    for pf in (0.1,0.3,0.5):
        print(f'ailenin kandirilma olasiligi {pf:.0%}:')
        for name,fm in (('5 uye ayni aile',[0]*5),('2 aile (3+2)',[0,0,0,1,1]),('3 aile (2+2+1)',[0,0,1,1,2]),('5 farkli aile',[0,1,2,3,4])):
            print(f'  {name:18s} onay esigi 3/5 -> ele gecirilme {p_capture(fm,need=3,p_family=pf):.1%} | 4/5 -> {p_capture(fm,need=4,p_family=pf):.1%}')
