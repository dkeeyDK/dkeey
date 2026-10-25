"""Bolum 42 kapanis olcutleri: kural mantigi simulasyonlari (model testi, gercek sistem degil). Sabit tohum."""
import random, math
R=random.Random(42)

# ---- A1: Kriz'de onay kilitlenmesi: eski (>=6/8 onay) vs yeni (>=3/7 veto durdurur) ----
def a1(n=20000):
    res={}
    for name in ('eski_6/8_onay','yeni_3/7_veto'):
        ok_crisis=ok_susp=0
        for _ in range(n):
            # Kriz: yalniz Watchdog + Cekirdek uyanik (7.3). Uyanik ajanlar onay verir; uyuyanlar veremez.
            awake=1; core=1
            if name.startswith('eski'): ok_crisis+= (awake+core)>=6
            else:
                # veto: hibernasyondaki ajanlar yalniz veto icin uyanir; durust veto olasiligi 3% (yanlis alarm)
                vetoes=sum(R.random()<0.03 for _ in range(7)); ok_crisis+= vetoes<3
            # iki ajan askida, kalan 5 ajan uyanik (normal mod)
            if name.startswith('eski'): app=sum(R.random()<0.97 for _ in range(5))+core; ok_susp+= app>=6
            else: vetoes=sum(R.random()<0.03 for _ in range(5)); ok_susp+= vetoes<3
        res[name]=(ok_crisis/n,ok_susp/n)
    # saldirgan: m ele gecirilmis ajan satisi durdurmak ister
    block={}
    for m in range(0,8):
        old = (7-m+1)<6   # eski: onaylayacak ajan+cekirdek 6'nin altina duserse durur
        new = m>=3
        block[m]=(old,new)
    return res,block
# ---- A5: sahte alarm: sayac disaridan beslenirse (eski) vs yalniz ajanin kendi eylemleri (yeni) ----
def a5(thr,days=365,runs=400,spam=0,eski=True):
    trig=0; safe_runs=0; susp_days=0
    for _ in range(runs):
        cool=0; hist=[]; safe=False
        for d in range(days):
            own=max(0,R.gauss(100,12)); counter=own+spam if eski else own
            if cool>0: cool-=1; continue
            if counter>thr and not safe:
                trig+=1; susp_days+=3; hist.append(d)
                if not eski:
                    cool=7
                    if len([x for x in hist if d-x<30])>=3: safe=True
        safe_runs+=safe
    return trig/runs, susp_days/runs, safe_runs/runs
# ---- B2: Sybil: cuzdan basi tavan vs ekonomik agirlik ----
def b2():
    out=[]
    total_weight=100.0
    for share in (5,10,20,30):
        row=[]
        for k in (1,4,10,50):
            # eski: cuzdan basi %5 tavan: k cuzdan -> min(share, 5*k)
            old=min(share,5*k)
            new=share        # agirlik = kilitli*sure, bolmek etkiyi degistirmez
            row.append((k,old,new))
        out.append((share,row))
    return out
# ---- B6: Uyku protokolu: gelir sifir ----
def b6(reserve=20800,ce=200,rev=0,weeks=140):
    r=reserve; w=0; sunset=None; user_withdraw_ok=True
    users_balance=1_000_000
    for w in range(weeks):
        runway=r/ce if ce else 99
        if sunset is None and runway<13 and rev<ce: sunset=w
        if sunset is None: r-=ce-rev if ce>rev else 0
        else: pass   # uykuda harcama yok (yalniz min. keeper) 
        # kullanici cekimi her zaman acik
        user_withdraw_ok = user_withdraw_ok and users_balance>=0
    return sunset,r,user_withdraw_ok
# ---- B7: panel ele gecirme: komite tek aile (p), panel farkli aileler ----
def b7(p=0.3,n=200000):
    out={}
    for size,thr in ((3,2),(5,4),(5,3),(7,5)):
        bad=0
        for _ in range(n):
            cap=sum(R.random()<p for _ in range(size))
            bad+= cap>=thr
        out[(size,thr)]=bad/n
    return out
if __name__=='__main__':
    res,block=a1()
    print('A1 Crisis ve iki ajan askida: onay/veto sonrasi satis acilma orani')
    for k,v in res.items(): print('  ',k,'Crisis:',round(v[0],3),'iki ajan askida:',round(v[1],3))
    print('  satisi durdurmak icin gereken ele gecirilmis ajan (eski/yeni):',[m for m in range(8) if block[m][0]][:1],[m for m in range(8) if block[m][1]][:1])
    for thr,lab in ((136,'3 sigma'),(148,'4 sigma')):
        for spam in (0,150,500):
            e=a5(thr,spam=spam,eski=True); y=a5(thr,spam=spam,eski=False)
            print(f'A5 esik {thr} ({lab}), disaridan spam {spam}/gun: askı tetigi/yil eski {e[0]:.1f} | yeni {y[0]:.1f}; yeni tasarimda guvenli moda dusen kosu orani {y[2]:.3f}')
    print('B2 (hisse, [(cuzdan sayisi, eski etki, yeni etki)]):')
    for share,row in b2(): print('  ',share,row)
    print('B6 Uyku (gelir 0):',b6(),' (gelir 100):',b6(rev=100))
    print('B7 panel ele gecirme olasiligi (aile basi p=0.3):',b7())
    print('B7 komite tek aile ve panel birlikte (p=.3):', {k:round(.3*v,4) for k,v in b7().items()})
