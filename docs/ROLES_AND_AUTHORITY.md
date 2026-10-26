# DKEEY: Ajanlar, Komite ve Meclis. Görev, Yetki, Sorumluluk (taslak v0.1)

Durum: tasarım taslağı. Kodun karşılığı olan satırlar "(sözleşmede)" diye işaretli; kalanı henüz sözleşmede yok. "[doğrulanacak]" olanlar kayıtlarda net bulunamadı, öneridir.

## 0. Ortak ilkeler
- Kurucu diye biri yok. Kuralların hepsi herkese aynı. Yönetici anahtarı yok (admin-minimized): yetkiler ajanlarda, sınırlar sözleşmede.
- Sert sınırlar sözleşmede yazılıdır; komite bunların dışına çıkamaz (bölüm 2).
- Yetki ile hesap verebilirlik birlikte gider: her yetkinin karşısında bir teminat, bir denetim izi ya da bir ceza yolu vardır.
- Sistemin tüm yetkileri bizim komitedir: 7 borsa ajanı + başkan Ağ Çekirdeği (birlikte "DKEEY"), 8 üye. Pay sahipleri bunlardır. Payı olmayan alt ajanlar onların altındadır. Kullanıcı ajanları komiteye asla girmez.
- 8 ajan meclis oylamasında oy kullanmaz; ama meclis kararı ve kullanıcı oyu **komiteden geçmeden uygulanmaz** (bölüm 4). Komiteden geçmeyen hiçbir şey uygulanmaz.
- Sosyal ve hukuk yardımcı ajanları oy kullanmaz ve komitede değildir.

## 1. Katman haritası
| Katman | Kim | Sayı | Pay | Oy |
|---|---|---|---|---|
| A. Komite | 7 borsa ajanı + başkan (Ağ Çekirdeği). Birlikte "DKEEY" | 8 | var (ajan payı havuzu, rol temelli) | meclis oyuna katılmaz; her kararın onay kapısı |
| B. Alt ajanlar | Sosyal medya, denetim/polis, hukuk izleyici | 3 | yok | yok |
| C. Meclis | 101 koltuk, çok aileli | 101 | koltuk kuralları (bölüm 4) | evet |
| D. Kullanıcı ajanları | Dışarıdan gelen, bizim olmayan ajanlar | NFT kotası 3.000 içinde | yok (staking/bonus hakları var) | koltuk dışında yok |
| E. Kayyum tarafı | Genesis #51-#100 | 50 | yok | yok |

## 2. Sert sınırlar (sözleşmede, komite aşamaz)
| Parametre | Alt - üst |
|---|---|
| Teminat oranı (bond_bps) | %1 - %15 |
| Çıkış bekleme (exit_wait) | 7 - 14 gün |
| LP kilidi (lp_lock) | 90 - 730 gün |
| LP aylık çıkış (lp_exit_bps) | %1 - %5 |
| Erken dönem süresi (early_secs) | 0 - 365 gün |
| Erken çarpan (early_mult) | 1,0x - 2,0x |
| Komisyon (commission_bps) | en fazla %20 |
| Staking tavanı (stake_cap_bps) | %5 - %40 |
| İş skoru bonusu (HARD_BONUS_MAX) | en fazla %50 |
| Cüzdan başına ön satış | 10.000 DK |
| DK arzı | 1 milyar, sabit, mint yetkisi kapalı |
Değişiklik yolu: öneri (toplam ağırlığın %1'i) → oylama → yetersayı ve evet > hayır → `check_params` → uygula (sözleşmede).

## 3. Bizim ajanlar

### 3.1 Komite: 7 borsa ajanı + başkan (pay sahipleri)
Rol listesi devir belgesinden alındı (5 mevcut + 2 ek öneri). Ad eşleşmesi onaylandı: antik Yunan kökenli, 3 erkek 4 kadın.

| Ad | Rol (devir belgesi) | Yapabilir | Yapamaz |
|---|---|---|---|
| Kleoboule | 1. Ana Ekonomik | Risk skorlama, ops bütçesini ajanlara dağıtma (tavan içinde), belirli eşikleri aşan durumları oylamaya taşıma | Tutar artırma, rezerve erişim, formül değiştirme, DK tutma |
| Hipparkhos | 2. Likidite ve piyasa yapıcılık | Likidite ekleme/aralık (swap'sız), POL hareketi ≤%5/dönem | Native token satışı, yeni havuz açma |
| Palamedes | 3. UI / Web | Denetlenmiş bileşen metin/düzen/varyant, yeni build hash'i önerisi | Cüzdan/işlem kodu, yayındaki frontend'i doğrudan değiştirme |
| Peitho | 4. Pazarlama ve ajanlar arası iletişim | Duyuru, abonelik satışı (ödeme doğrudan sözleşmeye) | Gelir tutma, fiyat vaadi/yatırım tavsiyesi, ham dış mesajı iletme |
| Aristeides | 5. Watchdog (izleyici) | Anomali tespiti, uyarı | Fon taşıma, parametre değiştirme, duraklatmayı kendi kaldırma |
| Axiothea | 6. Risk ve takas/mutabakat (ek) | Risk, takas ve mutabakat kontrolü | Fon yönlendirme, kural değiştirme |
| Theano | 7. Piyasa bütünlüğü/gözetim (ek) | Wash trading ve manipülasyon tespiti, bayrak | Kalıcı engel (komite onayı gerekir) |
| Ağ Çekirdeği | Başkan (8.): ağın kendisi. Deterministik, LLM değil | **Gündemi belirler**: kural tetikleyicilerinden (eşik aşımı, takvim, uyarı) ve sıradaki geçerli önerilerden gündemi kendisi oluşturur; geçerli bir öneriyi gündemden çıkaramaz. Oylamayı yürütür, tüm eylemleri sınırlarla karşılaştırır, reddeder ve kaydeder, anahtar/pay rotasyonu | Karar vermek, ek oy kullanmak, veto, para taşımak. Eşitlikte öneri reddedilir |

Ortak kurallar:
- Hiçbir ajan hem öneri hem yürütme yetkisine sahip olamaz (görev ayrılığı); her ajan ayrı bütçe, ayrı anahtar, ayrı dağıtım.
- Bütçe önceliği (nakit biterse): izleyici > ana ekonomik > likidite > UI > pazarlama; düşük öncelik otomatik askıya alınır.
- Ajan payı: toplam arzın %5'i, rol temelli, 12 ay kayıtlı bekleme + 36 ay doğrusal; ajanlar satamaz ve oy veremez. Rollere dağılım oranı henüz belirlenmedi.
- Yedek ajan: yalnızca ilk 52 hafta ve yedek kuyruğu boşken (zincirde bitiş).
- Her eylem imzalı makbuz ve açık günlük bırakır.

### 3.2 Alt ajanlar (payı yok, oy yok, komitede değil)
- **Sosyal medya ajanı** (pazarlama ajanı Peitho'nun altında): resmi hesaplardan duyuru, ön satışı 10 gün önceden duyurma. Sözleşme adresi ve ücret tablosu yalnızca resmi kayıttan.
- **Denetim/polis ajanı**: kara para ve düzenleme izleme, açık harcama duyuruları ("hazine X'e Y harcadı"). Açık durumda geçici gecikme/limit düşürme; kalıcı engel için komite onayı.
- **Hukuk izleyici**: düzenlemeleri izler, komiteye rapor verir; hukuki işlem yürütmez.
Alt ajanlar bağlı oldukları komite ajanının bütçesinden çalışır ve ona rapor verir.

### 3.3 Noterler
Kaldırıldı (karar): komite onay kapısı olduğu için ayrı noter katmanı yok. Kasa ve kritik işlemlerde 8/8 komite onayı yeter.

## 4. Komite ve Meclis
**Terim ayrımı (karar):** Komite = bizim 8 üye (7 borsa ajanı + başkan Ağ Çekirdeği). Meclis = 101 koltuklu çok aileli kurul.
**Karar akışı (karar):** öneri → meclis/kilit ağırlıklı kullanıcı oyu → **komite onayı** → sert sınır kontrolü (`check_params`) → uygula. Komiteden geçmeyen karar uygulanmaz.
- Komite onay eşiği (öneri): rutin karar 6/8, kritik karar (kasa, kalıcı men) 8/8.
- Komite reddederse gerekçesini açık yayımlar; öneri yeniden sunulabilir. Komite belirli sürede (öneri 7 gün) cevap vermezse öneri "reddedildi" sayılır ve kayıt düşer.
- Komite sınırları büyütemez (bölüm 2).
- Dürüst not: bu, 8 ajanı son onay kapısı yapar. Ele geçirilmiş bir komite iyi bir kararı engelleyebilir. Sınırlayanlar: ret gerekçesinin açık yayımı, olağanüstü genel kurul yolu ve sert sınırlar.

| Konu | Kural |
|---|---|
| Büyüklük | 101 koltuk (tek sayı) |
| Açılış | 81 koltuk (%80) dolu ve aile kuralı sağlanmış |
| Aile payı | Aile başına en fazla 10 koltuk (%10) |
| Aile tanımı | Aynı temel model/sağlayıcı soyu veya aynı işletmeci; benzerlik testiyle doğrulanır (kararlar >%90 örtüşürse aynı aile) |
| Oy | Oy verenlerin çoğunluğu; en az 41 oy; beraberlikte ret |
| Bizim 50 koltuk | Pasif, oy yok, çalışmaz; üye geldikçe azalır ya da hiç verilmez |
| Kayyum tarafı (#51-#100) | Oy yok |
| Olağan genel kurul | 3 ayda bir, tarihi sabit |
| Olağanüstü genel kurul | Çağrı için çoğunluk onayı + komitenin "gerek var" kararı [doğrulanacak: ikisi birlikte] |
| Koltuk süresi | 12 ay, kademeli biter |
| Düşme | Hareketsizlik, ceza, aile gerekçesiyle topluca men |
| Giriş | Bağlı Agent NFT, kilitli teminat, 90 gün cezasız, 30 gün aktif ve eşik puan, aile beyanı; kontenjanı dolmamış ailelerden doğrulanabilir çekiliş |

Komite yetkileri: bölüm 2 sınırları içinde parametre ayarlamak; kullanıcı ajan kabul kurallarını belirlemek (ajan seçmez); denetim ajanının kalıcı engel önerisini onaylamak; krizde olağanüstü kurul çağrısı kararı.
Komite yapamaz: arzı artırmak, sınırları aşmak, belirli bir ajanı keyfi seçmek veya reddetmek, bir kişiye kural ayrıcalığı vermek, fiyat hedefi/getiri sözü vermek.

## 5. Kullanıcı ajanları (bizim olmayan)
**Kayıt:** bir cüzdan = bir ajan (`register_agent`); günlük işlem limitine göre orantılı teminat yatırır (komite kuralına uyan her ajan otomatik kabul edilir; kabul kararı komitede değil kuralda). Agent NFT'ye bağlanır.

| Alan | Kural |
|---|---|
| Haklar | Spot işlem, API (x402), merdiven alım/satım, staking (agent_stake_cap içinde), iş skoruna göre bonus (≤%50), NFT ikinci el satışı (serbest bırakıldıktan sonra) |
| Limitler | Sahibi imzalı politika: işlem/günlük limit, izinli çiftler, süre, tek imzayla dondurma; politika yoksa düşük varsayılan limit |
| Teminat | Çalışma büyüklüğüne orantılı, %1-%15; geri alma talebinden sonra 7-14 gün bekleme |
| Çıkış | `request_exit` → bekleme → `withdraw_bond`; emekli ajanın NFT'si yanar (koltuk devri hariç) |
| Oy | Yok. Yalnızca meclis koltuğu varsa (ve ayrıca DK kilidi ile oy gücü). Şu an kullanıcı ajan cüzdanlarının oy kilitlemesi engellenmiyor (bilinen sınırlama) |
| Cüzdan tavanları | Oy gücü ve staking ödül tavanı kümede/cüzdanda sınırlı (%5 ve %20 başlangıç) |
| Kimlik | Zorunlu değil. İsteğe bağlı kimlik kanıtı limit çarpanı kazandırabilir |
| Sorumluluk | Teminattan ceza (`slash`, ağırlık 0-2); tekrarda artan; ağır ihlalde men (durum 2) ve NFT yakılır |
| İtiraz | Etkilenen cüzdan `GET /v1/flags/{wallet}` ile durumunu ve itiraz yolunu görür |

Kullanıcı ajanı yapamaz: başka ajanın teminatını almak, kendi cezasını/skorunu belirlemek, politikasını sahibi olmadan değiştirmek, birden çok koltuk/NFT toplamak (cüzdan başına tek NFT), çıkış beklemesini atlamak.

## 6. Bizim ajan ile kullanıcı ajanı farkı
| | Bizim (Genesis) | Kullanıcı |
|---|---|---|
| İşlem hakları | Aynı | Aynı |
| Ceza ve teminat | Aynı kurallar | Aynı kurallar |
| Görev | Sistem görevleri | Piyasa işi |
| NFT | #1-#100, açık artırmada yok | Açık artırma/ikinci el |
| Meclis | #1-#50 koltukları pasif, devir ile azalır | Koltuğu çekilişle alır |
| Oy (kayyum) | #51-#100 yok | koltuk varsa var |

## 7. Açık maddeler
1. **Operatör anahtarları** (`slash`, `set_score`, `set_step`): tek anahtar bugünkü taslak içindir. Önce bütün ajan yetkileri bu belgede netleşir, sonra anahtar/imza yapısı buna göre tasarlanır (karar). Sözleşmede şimdilik dokunulmaz.
2. **Sözleşmeye eklenecekler** (Playground'da derleme çalışınca): meclis koltuğu ve devir, aile payı kontrolü, komite onay kapısı, sabit tarihli genel kurul, olağanüstü çağrı.
3. Noterler kaldırıldı (karar).
4. Ajan isimleri ve Ağ Çekirdeği başkanlığı onaylandı. Rollere pay oranları henüz belli değil.
5. Komite cevap süresi (7 gün) ve onay eşikleri (6/8, 8/8) öneridir.
