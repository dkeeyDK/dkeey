import math
# Ayni toplam deger V ile: tam aralik (x*y=k) vs ajanin fiyat etrafina topladigi aralik [P/r, P*r] (v3 matematigi).
# Fiyat P=1, V=2 (tam aralikta X=1 token, Y=1 stable). Balina s token satar.
P=1.0; V=2.0
def L_range(r):
    if r is None: return V/(2*math.sqrt(P))
    pa=P/r; pb=P*r
    return V/(2*math.sqrt(P)-P/math.sqrt(pb)-math.sqrt(pa))
def sell(L,r,s):
    inv=1/math.sqrt(P)+s/L           # 1/sqrt(P')
    Pn=1/inv**2
    if r is not None and Pn<P/r: return None   # aralik tukendi
    return 1-Pn/P
Lf=L_range(None)
print("Ayni para (V=2): derinlik carpani ve balina satisinda fiyat dususu. s = tam-aralik havuzdaki token miktarinin kati.")
print(f"{'aralik':>10} | {'derinlik x':>10} | " + " | ".join(f"s={s:g}" .rjust(8) for s in (0.05,0.1,0.25,0.5,1.0)))
for name,r in (("tam",None),("±4x",4),("±2x",2),("±1.5x",1.5),("±1.25x",1.25),("±1.1x",1.1)):
    L=L_range(r); row=[]
    for s in (0.05,0.1,0.25,0.5,1.0):
        d=sell(L,r,s)
        row.append("aralik bitti" if d is None else f"%{d*100:5.0f}")
    print(f"{name:>10} | {L/Lf:10.1f} | " + " | ".join(x.rjust(8) for x in row))
