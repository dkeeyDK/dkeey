import math
# Havuz x*y=k. X=token, Y=stable, fiyat P=Y/X (baslangic 1.0), X0=1.0 birim.
# Balina W*X0 token satar, N parcaya boler, hizli ardarda (sybil/parca fark etmez: ucret havuz durumuna bagli).
# Degisken ucret = min(fmax, a * (kumulatif fiyat dususu)^2). Ucret havuza kalir (derinlik buyur).
def run(W,N,a,fmax,base=0.003):
    X=1.0;Y=1.0;k=X*Y;P0=1.0;proceeds=0;fees=0
    for _ in range(N):
        s=W/N
        P=Y/X; dd=max(0,1-P/P0)
        f=min(fmax, base + a*dd*dd)
        sin=s*(1-f)
        newX=X+sin; newY=k/newX if False else Y*X/newX
        out=Y-newY
        proceeds+=out
        X=X+s            # ucret de havuza kalir
        Y=newY
        k=X*Y
    P=Y/X
    return 1-P/P0, proceeds/(W*P0), (Y*1.0)
print("Balina W (havuz tokeninin kati) satar, 20 parca, hizli. 'net' = balinanin aldigi para / baslangic fiyatiyla deger")
print(f"{'W':>5} | {'ucret yok':>22} | {'dinamik ucret (a=3, max %25)':>32}")
for W in (0.1,0.25,0.5,1.0):
    d0,n0,_=run(W,20,0,0)
    d1,n1,_=run(W,20,3,0.25)
    print(f"{W:5.2f} | dusus %{d0*100:3.0f} net %{n0*100:3.0f} | dusus %{d1*100:3.0f} net %{n1*100:3.0f}")
