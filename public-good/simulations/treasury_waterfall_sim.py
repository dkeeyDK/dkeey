def L(s=""): print(s)
L("HAZINE SELALESI (waterfall) SIMULASYONU: once 'guvenlik tanki' dolar, tasan para ihtiyac havuzuna akar. 24 ay, aylik adim.")
L("TUM GIRDILER VARSAYIM (teklif/gercek hacim yok). Tank hedefi = 24 ay isletme gideri + denetim/hukuk rezervi A.")
L("Denetim/hukuk rezervi A = 50,000 USD varsayim (sektorde gorulen buyukluk; teklif ALINMADI).")
sc={ # raise, nft_price, monthly_volume, our_share, monthly_cost
 "DUSUK": dict(raise_=19_050,  nft=20,  vol=100_000,    share=0.0003, cost=97),
 "TEMEL": dict(raise_=38_100,  nft=50,  vol=1_000_000,  share=0.0005, cost=97),
 "YUKSEK":dict(raise_=190_500, nft=150, vol=10_000_000, share=0.0005, cost=378),
}
A=50_000; TREAS_FRAC=0.75; NFT_N=2900; NFT_MONTHS=12
L(f"\n {'senaryo':>7} {'tank hedefi':>11} | {'tank dolum ay':>13} | {'24 ay toplam gelir':>18} {'24 ay gider':>11} {'tasan (ihtiyac havuzu)':>22} | gelir kalemi payi: ön satis / NFT / parite")
for k,v in sc.items():
    target=24*v["cost"]+A
    tank=v["raise_"]*TREAS_FRAC; over=0; filled=None; tot_in=tank; tot_cost=0
    pre=tank; nft_t=0; par_t=0
    if tank>=target: filled=0; over+=tank-target; tank=target
    for m in range(1,25):
        nft=(NFT_N/NFT_MONTHS*v["nft"]) if m<=NFT_MONTHS else 0
        par=v["vol"]*v["share"]
        inflow=nft+par; nft_t+=nft; par_t+=par; tot_in+=inflow
        tank+=inflow-v["cost"]; tot_cost+=v["cost"]
        if tank>=target:
            over+=tank-target; tank=target
            if filled is None: filled=m
    tin=pre+nft_t+par_t
    L(f" {k:>7} {target:>11,.0f} | {('ay '+str(filled)) if filled is not None else 'dolmadi':>13} | {tot_in:>18,.0f} {tot_cost:>11,.0f} {over:>22,.0f} | {pre/tin:5.0%} / {nft_t/tin:5.0%} / {par_t/tin:5.0%}")
L("\nParite hacmi hassasiyeti (TEMEL, diger kalemler sabit): aylik hacim -> tank dolum ayi")
for vol in (50_000,200_000,1_000_000,5_000_000):
    v=dict(sc["TEMEL"]); v["vol"]=vol; target=24*v["cost"]+A; tank=v["raise_"]*TREAS_FRAC; filled=None
    for m in range(1,121):
        nft=(NFT_N/NFT_MONTHS*v["nft"]) if m<=NFT_MONTHS else 0
        tank+=nft+vol*v["share"]-v["cost"]
        if tank>=target: filled=m;break
    L(f"   hacim {vol:>9,}/ay -> {('ay '+str(filled)) if filled else '10 yilda bile dolmaz'}")
L("\nNFT geliri 12. aydan sonra biter; sonrasi icin yalniz parite komisyonu kalir. Haftalik cekim limiti (%0.5) tank->ihtiyac havuzu akisini da yavaslatir (v4 hazine kurali) - akis kurali ayri tasarlanmali.")
L("\nNFT satis orani hassasiyeti (TEMEL): NFT'lerin yuzde kaci satilirsa tank kac ayda dolar?")
for frac in (0.0,0.10,0.25,0.50,1.0):
    v=sc["TEMEL"]; target=24*v["cost"]+A; tank=v["raise_"]*TREAS_FRAC; filled=None
    for m in range(1,121):
        nft=(NFT_N*frac/NFT_MONTHS*v["nft"]) if m<=NFT_MONTHS else 0
        tank+=nft+v["vol"]*v["share"]-v["cost"]
        if tank>=target: filled=m;break
    L(f"   NFT satisi %{frac*100:>3.0f} -> {('ay '+str(filled)) if filled else '10 yilda bile dolmaz'}")
