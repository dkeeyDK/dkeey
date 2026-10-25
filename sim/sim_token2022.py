# Model: iki havuz A (bizim) ve B (baska yer), her biri toplam token derinliginin yarisi. Sabit carpim (x*y=k).
# Balina W token satar. Fiyat dususu = 1-(X/(X+s))^2. Arbitraj hizli varsayildi (havuzlar birlesik) ancak engellenmedigi surece.
c=0.10  # gunluk sinir: kayitli havuz tokeninin %10'u
def drop(X,s): return 1-(X/(X+s))**2
print("Toplam havuz tokeni X=1.0 (A=0.5, B=0.5). Balina W satar. Fiyat dususu = gun-1 sonu.")
print(f"{'W':>5} | {'Sinir yok':>9} | {'Sadece A siniri':>15} | {'Token2022 (%100 kayitli)':>24} | {'Token2022 (B kayitsiz)':>22}")
for W in (0.05,0.1,0.25,0.5,1.0):
    none=drop(1.0,W)                 # birlesik havuz
    venue=drop(0.5,W)                # A siniri: balina B'ye satar, B tek basina carpilir (arbitraj A'ya satamaz)
    tok=drop(1.0,min(W,c*1.0))       # tum likidite kayitli: toplam gunluk c*X
    tok_unreg=drop(0.5,W)            # B kayitsiz: sinir isliyor mu? hayir, B'ye sinirsiz satis
    days=W/(c*1.0)
    print(f"{W:5.2f} | %{none*100:7.0f}  | %{venue*100:13.0f}  | %{tok*100:11.0f} ({days:4.1f} gun)    | %{tok_unreg*100:12.0f}")
