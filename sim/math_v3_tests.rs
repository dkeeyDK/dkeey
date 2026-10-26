pub mod math {
    pub const MONTH: i64 = 30 * 86_400;
    pub const YEAR: i64 = 365 * 86_400;
    pub const SCALE: u128 = 1_000_000_000_000;

    // Ajan teminati: gunluk islem limitinin bps'i, alt sinir min.
    pub fn required_bond(daily_limit: u64, bps: u64, min: u64) -> Option<u64> {
        let v = (daily_limit as u128).checked_mul(bps as u128)? / 10_000;
        let v = u64::try_from(v).ok()?;
        Some(v.max(min))
    }

    // Kesinti: severity 0 uyari (yalniz ilk hata, strikes==0), 1 kucuk (%10), 2 ciddi (%100).
    // Donus: (kesilen, yeni strikes)
    pub fn slash_amount(bond: u64, severity: u8, strikes: u8) -> Option<(u64, u8)> {
        match severity {
            0 => if strikes == 0 { Some((0, 1)) } else { slash_amount(bond, 1, strikes) },
            1 => Some((((bond as u128) * 1_000 / 10_000) as u64, strikes.saturating_add(1))),
            2 => Some((bond, strikes.saturating_add(1))),
            _ => None,
        }
    }

    // ---- Kutu merdiveni (fiyat gecmisi hist[i] = i. kutunun fiyati; geri satis tam ayni fiyatlardan geri iner) ----
    pub fn next_price(p: u64, step_bps: u64) -> u64 {
        if step_bps == 0 { return p; }
        let np = ((p as u128) * (10_000 + step_bps) as u128 / 10_000) as u64;
        np.max(p + 1)
    }
    // Alim: en fazla max_bins kutu. Donus: (maliyet, yeni_idx, yeni_bin_sold, doldurulan_dk)
    pub fn ladder_buy(hist: &mut [u64], idx: u64, bin_size: u64, bin_sold: u64, step_bps: u64, dk: u64, max_bins: u64)
        -> Option<(u64, u64, u64, u64)> {
        if bin_size == 0 || bin_sold >= bin_size || (idx as usize) >= hist.len() || hist[idx as usize] == 0 { return None; }
        let (mut i, mut sold, mut rem) = (idx, bin_sold, dk);
        let (mut cost, mut filled, mut bins) = (0u128, 0u64, 0u64);
        while rem > 0 && bins < max_bins {
            let p = hist[i as usize];
            let take = rem.min(bin_size - sold);
            cost = cost.checked_add((take as u128).checked_mul(p as u128)?)?;
            filled = filled.checked_add(take)?; sold += take; rem -= take;
            if sold == bin_size {
                if (i as usize) + 1 >= hist.len() { break; }
                hist[i as usize + 1] = next_price(p, step_bps);
                i += 1; sold = 0; bins += 1;
            }
        }
        Some((u64::try_from(cost).ok()?, i, sold, filled))
    }
    // Geri satis: asla piyasa fiyatinin ustunde degil (her kutu kendi alim fiyatindan, spread dusulur). Donus: (gelir, yeni_idx, yeni_bin_sold, satilan_dk)
    pub fn ladder_sell(hist: &[u64], idx: u64, bin_size: u64, bin_sold: u64, dk: u64, spread_bps: u64, max_bins: u64)
        -> Option<(u64, u64, u64, u64)> {
        if bin_size == 0 || spread_bps > 10_000 || (idx as usize) >= hist.len() { return None; }
        let (mut i, mut sold, mut rem) = (idx, bin_sold, dk);
        let (mut out, mut filled, mut bins) = (0u128, 0u64, 0u64);
        while rem > 0 {
            if sold == 0 {
                if i == 0 || bins >= max_bins { break; }
                i -= 1; sold = bin_size; bins += 1; continue;
            }
            let take = rem.min(sold);
            let v = (take as u128).checked_mul(hist[i as usize] as u128)?.checked_mul((10_000 - spread_bps) as u128)? / 10_000;
            out = out.checked_add(v)?;
            sold -= take; rem -= take; filled += take;
        }
        Some((u64::try_from(out).ok()?, i, sold, filled))
    }
    // Tur (orantili dolum): talep > kapasite ise herkes ayni oranda dolar (asagi yuvarlanir, toplam <= filled).
    pub fn pro_rata(dk: u64, filled: u64, demand: u64) -> u64 {
        if demand == 0 { 0 } else { ((dk as u128) * filled as u128 / demand as u128) as u64 }
    }

    // ---- On satis: adil (esit) dagitim. Istek buyuklugune gore 128 kova (ust sinir = max*(k+1)^2/16384). ----
    pub const BUCKETS: usize = 128;
    pub fn bucket_bound(max_req: u64, k: usize) -> u64 {
        ((max_req as u128) * ((k as u128 + 1) * (k as u128 + 1)) / 16_384) as u64
    }
    pub fn bucket_of(max_req: u64, req: u64) -> Option<usize> {
        if req == 0 || req > max_req { return None; }
        for k in 0..BUCKETS { if req <= bucket_bound(max_req, k) { return Some(k); } }
        None
    }
    // Su-doldurma: herkes min(istek, seviye) alir. Once kova sinirinda en yuksek uygun seviye bulunur (toplam TAM bilinir),
    // sonra artan oda, ustte kalan isteklere esit eklenir; ek, ustteki kovalarin (guvenli) en kucuk istegini asmaz,
    // boylece toplam yine TAM bilinir. minr[k] yalniz asagi guncellenir (eski kucuk deger de guvenlidir).
    // Donus: (seviye, dagitilan). Talep <= room ise (u64::MAX, talep).
    pub fn fair_level(room: u64, max_req: u64, cnt: &[u32; 128], sum: &[u64; 128], minr: &[u64; 128]) -> (u64, u64) {
        let total: u128 = sum.iter().map(|x| *x as u128).sum();
        if total <= room as u128 { return (u64::MAX, total as u64); }
        let n_total: u128 = cnt.iter().map(|x| *x as u128).sum();
        let (mut below, mut n_below) = (0u128, 0u128);
        let mut best: (u128, u128, i32) = (0, 0, -1); // (seviye, toplam, kova)
        for k in 0..BUCKETS {
            below += sum[k] as u128; n_below += cnt[k] as u128;
            let l = bucket_bound(max_req, k) as u128;
            let f = below + l * (n_total - n_below);
            if f <= room as u128 { best = (l, f, k as i32); } else { break; }
        }
        let (l, f, kb) = best;
        let n_above: u128 = cnt.iter().enumerate().filter(|(j, _)| (*j as i32) > kb).map(|(_, c)| *c as u128).sum();
        if n_above == 0 { return (l as u64, f as u64); }
        let min_above: u128 = (0..BUCKETS).filter(|j| (*j as i32) > kb && cnt[*j] > 0).map(|j| minr[j] as u128).min().unwrap_or(l);
        let extra = ((room as u128 - f) / n_above).min(min_above.saturating_sub(l));
        ((l + extra) as u64, (f + extra * n_above) as u64)
    }

    // ---- Komite oylamasi ----
    pub fn vote_weight(amount: u64, secs: i64) -> u128 { (amount as u128) * (secs.max(0) as u128) / 86_400 }
    pub fn eff_weight(w: u128, snap_total: u128, cap_bps: u64) -> u128 { w.min(snap_total * cap_bps as u128 / 10_000) }
    pub fn passes(yes: u128, no: u128, snap_total: u128, quorum_bps: u64) -> bool {
        yes > no && yes * 10_000 >= snap_total * quorum_bps as u128
    }

    // Kilitli likidite: NAV'a gore pay. nav = havuzdaki ana para.
    pub fn shares_for(amount: u64, total_shares: u64, nav: u64) -> Option<u64> {
        if total_shares == 0 || nav == 0 { return Some(amount); }
        u64::try_from((amount as u128).checked_mul(total_shares as u128)? / nav as u128).ok()
    }
    pub fn value_for(shares: u64, total_shares: u64, nav: u64) -> Option<u64> {
        if total_shares == 0 { return None; }
        u64::try_from((shares as u128).checked_mul(nav as u128)? / total_shares as u128).ok()
    }
    // Erken donem ucret carpani (bps): ilk early_secs boyunca early_mult, sonraki early_secs boyunca 10000'e dogrusal iner.
    pub fn mult_bps(now: i64, start: i64, early_secs: i64, early_mult: u64) -> u64 {
        if early_secs <= 0 || early_mult <= 10_000 { return 10_000; }
        let t = now - start;
        if t <= early_secs { early_mult }
        else if t >= 2 * early_secs { 10_000 }
        else {
            let extra = (early_mult - 10_000) as i128;
            let left = (2 * early_secs - t) as i128;
            (10_000 + extra * left / early_secs as i128) as u64
        }
    }
    // Komisyon: (komisyon, kalan)
    pub fn split_commission(amount: u64, bps: u64) -> (u64, u64) {
        let fee = ((amount as u128) * bps as u128 / 10_000) as u64;
        (fee, amount - fee)
    }
    // Kilit bittikten sonra ayda init_shares * exit_bps kadar cikis hakki birikir (ilk ay kilit bitince acilir).
    pub fn allowed_exit(init_shares: u64, exit_bps: u64, lock_until: i64, now: i64) -> u64 {
        if now < lock_until { return 0; }
        let months = ((now - lock_until) / MONTH + 1) as u128;
        let v = (init_shares as u128) * exit_bps as u128 * months / 10_000;
        v.min(init_shares as u128) as u64
    }
    pub fn pending_fee(weight: u128, acc: u128, debt: u128) -> u64 {
        let gross = weight.saturating_mul(acc) / SCALE;
        gross.saturating_sub(debt) as u64
    }

    // Staking: tavan oraninda yillik odul; ajan bonusu yalniz agent_cap'e kadar olan kisma, tavan oraninin bonus_bps'i kadar eklenir.
    pub fn stake_accrue(amount: u64, agent_cap: u64, cap_bps: u64, bonus_bps: u64, dt: i64) -> u64 {
        if dt <= 0 { return 0; }
        let base = (amount as u128) * cap_bps as u128 * dt as u128 / (YEAR as u128 * 10_000);
        let elig = amount.min(agent_cap) as u128;
        let bonus = elig * cap_bps as u128 * bonus_bps as u128 * dt as u128 / (YEAR as u128 * 10_000 * 10_000);
        (base + bonus).min(u64::MAX as u128) as u64
    }
    pub const YEAR_BUDGET: [u64; 5] = [75_000_000, 62_500_000, 50_000_000, 37_500_000, 25_000_000];
    pub fn year_index(now: i64, start: i64) -> Option<usize> {
        if now < start { return None; }
        let y = ((now - start) / YEAR) as usize;
        if y < 5 { Some(y) } else { None }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test] fn bond() {
            assert_eq!(required_bond(1_000_000, 500, 50_000), Some(50_000));
            assert_eq!(required_bond(100_000_000, 500, 50_000), Some(5_000_000));
            assert_eq!(required_bond(u64::MAX, 10_000, 1), Some(u64::MAX));
        }
        #[test] fn slash() {
            assert_eq!(slash_amount(1000, 0, 0), Some((0, 1)));      // ilk kucuk teknik hata: uyari
            assert_eq!(slash_amount(1000, 0, 1), Some((100, 2)));    // tekrar: %10
            assert_eq!(slash_amount(1000, 1, 0), Some((100, 1)));
            assert_eq!(slash_amount(1000, 2, 0), Some((1000, 1)));
            assert_eq!(slash_amount(1000, 9, 0), None);
        }
        fn lcg(s: &mut u64) -> u64 { *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); *s >> 33 }
        #[test] fn ladder() {
            let mut h = [0u64; 128]; h[0] = 1000;
            // 5 kutu bir islemde
            let r = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 10_000_000, 5).unwrap();
            assert_eq!(r.3, 5_000_000); assert_eq!(r.1, 5); assert_eq!(r.2, 0);
            assert!(h[5] > 1000 && h[5] < 1200);
            assert!(r.0 > 5_000_000_000 && r.0 < 6_000_000_000);
            // kutu icinde sabit fiyat
            let mut h = [0u64; 128]; h[0] = 1000;
            let r = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 1_000, 5).unwrap();
            assert_eq!((r.0, r.1, r.2), (1_000_000, 0, 1000));
            // 50 kutu sonunda ~5 kat
            let mut h = [0u64; 128]; h[0] = 1000; let (mut i, mut s) = (0u64, 0u64);
            for _ in 0..50 { let r = ladder_buy(&mut h, i, 1_000_000, s, 330, 1_000_000, 5).unwrap(); i = r.1; s = r.2; }
            assert!(h[i as usize] > 4500 && h[i as usize] < 5500, "{}", h[i as usize]);
            // adim 0: fiyat sabit
            let mut h = [0u64; 128]; h[0] = 1000;
            assert_eq!(ladder_buy(&mut h, 0, 10, 0, 0, 10, 5).unwrap().1, 1);
            assert_eq!(h[1], 1000);
            // gecersiz
            let mut h = [0u64; 128];
            assert!(ladder_buy(&mut h, 0, 10, 0, 10, 5, 5).is_none());
        }
        #[test] fn ladder_sell_roundtrip() {
            let mut h = [0u64; 128]; h[0] = 1000;
            // 3.5 kutu al, hepsini geri sat: gelir <= odenen (spread kadar eksik), asla fazla degil
            let b = ladder_buy(&mut h, 0, 1_000_000, 0, 330, 3_500_000, 8).unwrap();
            let s = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 50, 8).unwrap();
            assert_eq!(s.3, 3_500_000); assert_eq!((s.1, s.2), (0, 0));
            assert!(s.0 <= b.0, "{} {}", s.0, b.0);
            assert!(s.0 >= b.0 * 99 / 100, "spread cok yuksek");
            // spread 0: tam esit (fiyat gecmisi birebir geri iner)
            let s0 = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 0, 8).unwrap();
            assert_eq!(s0.0, b.0);
            // kismi satis: kutu icinde
            let p = ladder_sell(&h, b.1, 1_000_000, b.2, 100, 0, 8).unwrap();
            assert_eq!((p.1, p.2, p.3), (b.1, b.2 - 100, 100));
            // sadece baslangicta: satilacak bir sey yok
            let z = ladder_sell(&h, 0, 1_000_000, 0, 10, 50, 8).unwrap();
            assert_eq!((z.0, z.3), (0, 0));
            // max_bins siniri
            let m = ladder_sell(&h, b.1, 1_000_000, b.2, 3_500_000, 0, 1).unwrap();
            assert!(m.3 < 3_500_000);
            // simulasyon: rastgele al-sat, kasa asla negatif, sold_total == idx*bin+sold
            let mut h = [0u64; 128]; h[0] = 1000; let (mut i, mut so) = (0u64, 0u64);
            let (mut paid, mut got) = (0u128, 0u128); let mut seed = 7u64; let mut tot: u64 = 0;
            for _ in 0..4000 {
                let amt = lcg(&mut seed) % 2_500_000 + 1;
                if lcg(&mut seed) % 2 == 0 {
                    if let Some(r) = ladder_buy(&mut h, i, 1_000_000, so, 330, amt, 5) { if r.3 > 0 { paid += r.0 as u128; i = r.1; so = r.2; tot += r.3; } }
                } else if let Some(r) = ladder_sell(&h, i, 1_000_000, so, amt.min(tot), 50, 5) { got += r.0 as u128; i = r.1; so = r.2; tot -= r.3; }
                assert_eq!(tot, i * 1_000_000 + so);
                assert!(got <= paid, "havuz acik verdi");
                if i >= 120 { break; }
            }
        }
        #[test] fn prorata() {
            // talep 3x kapasite: herkes 1/3
            assert_eq!(pro_rata(300, 1000, 3000), 100);
            assert_eq!(pro_rata(7, 1000, 3000), 2);
            let mut seed = 3u64;
            for _ in 0..200 {
                let n = (lcg(&mut seed) % 40 + 1) as usize;
                let o: Vec<u64> = (0..n).map(|_| lcg(&mut seed) % 100_000 + 1).collect();
                let d: u64 = o.iter().sum(); let cap = lcg(&mut seed) % d + 1;
                let f: u64 = o.iter().map(|x| pro_rata(*x, cap, d)).sum();
                assert!(f <= cap && cap - f < n as u64 + 1);
            }
            assert_eq!(pro_rata(5, 10, 0), 0);
        }
        #[test] fn fair() {
            let max_req = 10_000u64 * 1000; // 10K DK x 1000 lamport
            // bucket
            assert_eq!(bucket_of(max_req, max_req), Some(127));
            assert_eq!(bucket_of(max_req, 1), Some(0));
            assert_eq!(bucket_of(max_req, 0), None);
            assert_eq!(bucket_of(max_req, max_req + 1), None);
            for k in 0..BUCKETS { assert_eq!(bucket_of(max_req, bucket_bound(max_req, k)), Some(k)); }
            // rastgele: tam esitlik ve eksiksiz hesap
            let mut seed = 99u64; let mut worst_gap = 0f64;
            for round in 0..300 {
                let n = (lcg(&mut seed) % 300 + 1) as usize;
                let mut reqs: Vec<u64> = Vec::new();
                for _ in 0..n {
                    let r = match lcg(&mut seed) % 4 { 0 => max_req, 1 => max_req / 2, _ => lcg(&mut seed) % max_req + 1 };
                    reqs.push(r.max(1));
                }
                let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128];
                for r in &reqs { let b = bucket_of(max_req, *r).unwrap(); cnt[b] += 1; sum[b] += *r; minr[b] = minr[b].min(*r); }
                let total: u64 = reqs.iter().sum();
                let room = if round % 3 == 0 { total } else { (total as u128 * (lcg(&mut seed) % 95 + 1) as u128 / 100) as u64 };
                let (lv, al) = fair_level(room, max_req, &cnt, &sum, &minr);
                let real: u64 = reqs.iter().map(|r| (*r).min(lv)).sum();
                assert_eq!(real, al, "toplam tam bilinmeli");
                assert!(al <= room);
                // kesin su-doldurma seviyesi
                let mut lo = 0u64; let mut hi = max_req;
                while lo < hi { let mid = (lo + hi + 1) / 2; let f: u128 = reqs.iter().map(|r| (*r).min(mid) as u128).sum(); if f <= room as u128 { lo = mid } else { hi = mid - 1 } }
                if total > room {
                    let exact: u64 = reqs.iter().map(|r| (*r).min(lo)).sum();
                    assert!(lv <= lo || lv == u64::MAX);
                    if n >= 30 { worst_gap = worst_gap.max((exact - al) as f64 / room.max(1) as f64); }
                }
                // en buyuk cuzdan en kucukten (esit seviye) fazla alamaz
                if let (Some(mx), true) = (reqs.iter().max(), lv != u64::MAX) { assert!((*mx).min(lv) <= lv); }
            }
            assert!(worst_gap < 0.06, "kova kaba kaldi: {}", worst_gap);
            // balina testi: 1 balina (max) + 200 kucuk (50 DK), oda = %20 balina+kucuk
            let small = 50u64 * 1000;
            let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128];
            let bw = bucket_of(max_req, max_req).unwrap(); cnt[bw] += 1; sum[bw] += max_req; minr[bw] = max_req;
            let bs = bucket_of(max_req, small).unwrap(); cnt[bs] += 200; sum[bs] += 200 * small; minr[bs] = small;
            let room = 12_000_000u64; // talep 20M (balina 10M + 200x50K), oda 12M
            let (lv, al) = fair_level(room, max_req, &cnt, &sum, &minr);
            assert!(al <= room && lv >= small, "kucukler tam dolmali: lv={}", lv);
            let whale = max_req.min(lv);
            assert!(whale <= 2_000_000 && al == whale + 200 * small, "balina yalniz artan odayi alir: {}", whale);
            // az talep: hepsi tam
            assert_eq!(fair_level(u64::MAX, max_req, &cnt, &sum, &minr).0, u64::MAX);
            // cok kalabalik: seviye 0
            let mut cnt = [0u32; 128]; let mut sum = [0u64; 128]; let mut minr = [u64::MAX; 128]; cnt[0] = 1000; sum[0] = 1000 * 9000; minr[0] = 9000;
            let r = fair_level(100, max_req, &cnt, &sum, &minr); assert_eq!(r, (0, 0)); // 100 lamport / 1000 cuzdan = 0 (tam bolunmez)
            let r = fair_level(5000, max_req, &cnt, &sum, &minr); assert_eq!(r, (5, 5000)); // 5 lamport kisi basi
        }
        #[test] fn gov() {
            let w = vote_weight(1_000_000, 365 * 86_400); assert_eq!(w, 365_000_000);
            assert_eq!(vote_weight(10, -5), 0);
            assert_eq!(eff_weight(500, 1000, 500), 50);      // %5 tavan
            assert_eq!(eff_weight(10, 1000, 500), 10);
            assert!(passes(60, 40, 1000, 500));              // yes>no ve kenar yok
            assert!(!passes(40, 40, 1000, 500));
            assert!(!passes(30, 10, 1000, 500));             // yes>no ama yetersiz katilim (30 < 50)
            assert!(passes(50, 0, 1000, 500));
            // 100 cuzdan bolunmus balina: her biri tavan altinda -> toplam agirlik korunur (bilinen sinir: kume tespiti zincir disi)
            let split: u128 = (0..100).map(|_| eff_weight(30, 1000, 500)).sum(); assert_eq!(split, 3000);
        }
        #[test] fn lp() {
            assert_eq!(shares_for(100, 0, 0), Some(100));
            assert_eq!(shares_for(150, 100, 150), Some(100)); // NAV 1.5 -> ayni para daha az pay
            assert_eq!(value_for(100, 200, 300), Some(150));
            assert_eq!(split_commission(1000, 700), (70, 930));
            assert_eq!(mult_bps(0, 0, 100, 15_000), 15_000);
            assert_eq!(mult_bps(100, 0, 100, 15_000), 15_000);
            assert_eq!(mult_bps(150, 0, 100, 15_000), 12_500);
            assert_eq!(mult_bps(200, 0, 100, 15_000), 10_000);
            assert_eq!(mult_bps(999, 0, 100, 15_000), 10_000);
            assert_eq!(allowed_exit(1000, 200, 1000, 999), 0);          // kilit suruyor
            assert_eq!(allowed_exit(1000, 200, 1000, 1000), 20);        // ilk ay %2
            assert_eq!(allowed_exit(1000, 200, 1000, 1000 + MONTH), 40);
            assert_eq!(allowed_exit(1000, 200, 1000, 1000 + 100 * MONTH), 1000); // tavan
            assert_eq!(pending_fee(100, 2 * SCALE, 50), 150);
        }
        #[test] fn stake() {
            // 1M stake, %20 tavan, 1 yil: 200K
            assert_eq!(stake_accrue(1_000_000, 100_000, 2000, 0, YEAR), 200_000);
            // ajan bonusu %50: yalniz 100K'ya kadar kisma 20%*50% = ek 10K
            assert_eq!(stake_accrue(1_000_000, 100_000, 2000, 5000, YEAR), 210_000);
            // tum stake uygunsa %30
            assert_eq!(stake_accrue(100_000, 100_000, 2000, 5000, YEAR), 30_000);
            assert_eq!(stake_accrue(5, 5, 2000, 5000, -1), 0);
            assert_eq!(year_index(0, 0), Some(0));
            assert_eq!(year_index(YEAR, 0), Some(1));
            assert_eq!(year_index(5 * YEAR, 0), None);
            assert_eq!(year_index(-1, 0), None);
        }
    }
}
