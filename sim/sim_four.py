fee=0.003; base=40e6; V=100e6
def run(g, vol0=20e6, years=12):
    cum1=cum_o=0; out={}
    p1=po=None
    for m in range(1,years*12+1):
        vol=vol0*(1+g)**(m/12)
        if m<=12: tvl=base+V; s1=V/tvl; so=0
        else: tvl=base+4*V; s1=V/tvl; so=V/tvl
        inc=vol*fee*30
        cum1+=inc*s1
        if m>12: cum_o+=inc*so
        if p1 is None and cum1>=V: p1=m/12
        if po is None and m>12 and cum_o>=V: po=(m-12)/12
        if m in (12,24,36,60): out[m]=(cum1/V*100, cum_o/V*100)
    return p1,po,out
print("Yil 1: 1 yatirimci (100M USD). Yil 2 basinda 3 yatirimci daha (her biri 100M USD). Mevcut havuz 40M USD.")
print("Pay (NAV'a gore): yil 1 -> yatirimci1 %71 | yil 2 -> her yatirimci %23 (4x), mevcut havuz %9")
print("Ucret getirisi sermaye basina: yil 1 %15.6 -> yil 2 %5.0 (hacim sabit 20M USD/gun)")
print()
print(f"{'hacim buyumesi/yil':>20} | {'Yatirimci 1 geri alim':>22} | {'Yatirimci 2-4 geri alim':>24}")
for g in (0,0.5,1.0,2.0):
    p1,po,o=run(g)
    f=lambda x:('%.1f yil'%x) if x else '>12 yil'
    print(f"{g*100:19.0f}% | {f(p1):>22} | {f(po)+' (girisinden)':>24}")
print("\nBirikmis ucret geliri (yatirimin %'si): 1.yil sonu | 2.yil sonu | 3.yil sonu | 5.yil sonu")
for g in (0,0.5,1.0,2.0):
    p1,po,o=run(g)
    print(f"  hacim buyume %{g*100:3.0f}: Y1: %{o[12][0]:.0f}, %{o[24][0]:.0f}, %{o[36][0]:.0f}, %{o[60][0]:.0f} | Y2-4: -, %{o[24][1]:.0f}, %{o[36][1]:.0f}, %{o[60][1]:.0f}")
