import math
fee=0.003; V=100e6; base=40e6
def payback(vol0,g,tvl_fn,months=240,share_fn=None):
    cum=0
    for m in range(1,months+1):
        vol=vol0*(1+g)**(m/12)
        tvl=tvl_fn(m); share=V/tvl
        cum+=vol*fee*share*30
        if cum>=V: return m/12
    return None
print("1) Ortak hic cikmiyor, baska LP gelmiyor (havuz 140M USD, ortak payi %71). Gunluk hacim 20M USD'den buyuyor.")
print("   Yatirimi geri alma suresi (sadece ucret geliri, kayip ve fiyat degisimi haric):")
for g in (0,0.5,1.0,2.0):
    r=payback(20e6,g,lambda m:base+V)
    print(f"   hacim yillik buyume %{g*100:3.0f}: {('%.1f yil'%r) if r else '20 yildan uzun'}")
print("\n   Hacim 5M USD/gun ise (dusuk):")
for g in (0,1.0):
    r=payback(5e6,g,lambda m:base+V)
    print(f"   hacim yillik buyume %{g*100:3.0f}: {('%.1f yil'%r) if r else '20 yildan uzun'}")
print("\n2) Ikinci yatirimci 3. ayda ayni 100M USD ile giriyor. Pay token/NAV fiyatindan verilirse:")
vol=20e6
tvl1=base+V; tvl2=base+2*V
y1=vol*fee*365/tvl1; y2=vol*fee*365/tvl2
print(f"   Ucret getirisi her 1 USD sermayeye: once %{y1*100:.1f}/yil, sonra %{y2*100:.1f}/yil (ayni pasta daha cok sermaye arasinda bolunur)")
print(f"   Ilk ortak payi: %{100*V/tvl1:.0f} -> %{100*V/tvl2:.0f}; ikinci ortak payi %{100*V/tvl2:.0f}")
print(f"   Geri alma suresi (hacim sabit): ilk ortak 3 ay ayni, sonra {1/y2:.1f} yil; ikinci ortak {1/y2:.1f} yil")
print("   Hacim 2 kat olursa getiri 2 kat; sermaye girisi getiriyi ayni oranda dusurur. Bu yuzden sermaye getiri piyasa seviyesine inene kadar girer.")
