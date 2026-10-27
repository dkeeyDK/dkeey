import numpy as np
def L(s=""): print(s)
ROUNDS=1440; ORD=0.3; TARGET=30
BLOCK_CU=50e6; BLOCKS_DAY=216_000           # ASSUMPTION: block CU limit is of this order and changes; verify before design freeze
FEE_BASE=5_000                               # lamports per signature
L("POOL MALIYET / KAPASITE SIMULASYONU (gizli havuz: her harcama/cekim = 1 Groth16 dogrulamasi)")
L(" Varsayim: dogrulama 200K / 350K / 500K CU (kaynak: Chainstack 170-500K); 1 islem = 1 kanit (islem boyutu limiti 1232 bayt); cover islemi de kanit ister.")
L(" Havuzdaki kullanici basina 0.3 islem/gun (islem = harcama => 1 kanit); her 10 islemde 1 cekim (1 ek kanit). Tepe yuk = ortalama x 50.")
L(f" {'kullanici':>9} {'havuz%':>6} {'havuz kul.':>10} {'gercek/taraf/tur':>16} {'ortu cifti/tur':>14} {'kanit/gun':>10} {'tx/sn ort':>9} {'tx/sn tepe':>10} {'zincir kap. %(350K)':>20} {'ucret SOL/gun':>13} {'seviye':>7}")
for N in (10_000,100_000,1_000_000):
    for f in (0.01,0.10,0.30):
        U=N*f; real=U*ORD/ROUNDS/2
        cover=max(0,TARGET-real)                      # pairs per round
        proofs_real=U*ORD*1.1; proofs_cov=cover*2*ROUNDS
        proofs=proofs_real+proofs_cov
        cu=proofs*350e3; cap=cu/(BLOCK_CU*BLOCKS_DAY)
        fee=proofs*FEE_BASE/1e9
        tx_avg=proofs/86400; tx_peak=tx_avg*50
        s=real+cover; lvl="YUKSEK" if s>=20 else "ORTA" if s>=5 else "DUSUK"
        L(f" {N:>9,} {f:>6.0%} {U:>10,.0f} {real:>16.2f} {cover:>14.1f} {proofs:>10,.0f} {tx_avg:>9.1f} {tx_peak:>10.1f} {cap:>19.4%} {fee:>13.2f} {lvl:>7}")
L("\n Dogrulama CU duyarliligi (N=100,000, havuz %10): ")
N,f=100_000,0.10; U=N*f; real=U*ORD/ROUNDS/2; cover=max(0,TARGET-real); proofs=U*ORD*1.1+cover*2*ROUNDS
for cu in (200e3,350e3,500e3):
    L(f"   {cu/1e3:>4.0f}K CU/kanit -> gunluk toplam {proofs*cu/1e9:6.1f} milyar CU = zincir kapasitesinin {proofs*cu/(BLOCK_CU*BLOCKS_DAY):.4%}'u")
L("\n Gecikme: kanit olusturma (kullanici cihazinda) + 1 islem onayi. Kullaniciyi bekletmez ama telefonda saniyeler surebilir (olculmedi).")
L(" Merkle agaci: derinlik 20 = 1,048,576 not; 1M kullanici x birden cok not icin derinlik 24-26 gerekir. Her ekleme derinlik kadar Poseidon hash'i = ek CU (OLCULMEDI).")
L(" Tek islemde birden fazla kanit: kanit 256-388 bayt, islem 1232 bayt -> islem basina 1-2 kanit; tum tur icin tek birlesik kanit (rollup tarzi) ayri arastirma.")
L("\nNOT: tum degerler tahmin; bloktaki CU limiti, ucret ve Poseidon maliyeti gercek olcumle degisir. Kanit uretim suresi ve gizli kurulum (trusted setup) maliyeti modellenmedi.")
