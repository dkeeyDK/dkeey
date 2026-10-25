# DKEEY Ajan API Şartnamesi (taslak v0.1)

Durum: tasarım taslağı. Kodlanmadı, test edilmedi. Amaç: yapay zekâ ajanının kimlik göstermeden, emanetçi olmayan biçimde, tek çağrılarla spot alım-satım yapabilmesi.

## 1. İlkeler
- Hesap yok. Kimlik = cüzdan anahtarı. Her istek cüzdan imzasıyla gelir.
- Emanetçi olmayan: para sözleşmedeki kasada durur, API para tutmaz, taşıyamaz.
- Ajan sahibinin sınırları sözleşmede yazılıdır (bölüm 4). API bu sınırları aşan emri sözleşme seviyesinde reddeder.
- Tek fiyatlı toplu müzayede: emir sırası ya da hız avantaj sağlamaz (bölüm 3).
- Her yanıt imzalı makbuz taşır, makbuzların özeti zincire yazılır (denetim izi).
- İsteğe bağlı kimlik: ajan isterse bir kimlik kanıtı (ör. ERC-8004 / KYA belirteci) ekleyebilir. Zorunlu değildir. Eklenirse limitler yükselebilir (bölüm 4).

## 2. Kimlik doğrulama ve ödeme
- Başlıklar: `X-DK-Wallet` (public key), `X-DK-Nonce` (artan sayı), `X-DK-Sig` (ed25519; istek gövdesi + nonce + yol üzerinde).
- Tekrar saldırısı: nonce her cüzdan için kesin artan olmalı, aksi halde `409 nonce_reused`.
- Okuma uçları ücretsiz (hız sınırlı). Yazma ve yoğun okuma uçları x402 ile ödenir: sunucu `402 Payment Required` ile fiyat ve alıcı adres döner, ajan stablecoin mikro ödemesiyle tekrar dener. Ödeme tutarı işlem değerinden bağımsız, sabit küçük tutardır.
- Ücret tablosu ve alıcı adres resmi adres kaydında yayımlanır (sosyal duyurularla aynı güven çapası).

## 3. Emir akışı (toplu müzayede)
Zaman, ardışık "parti" (batch) pencerelerine bölünür. Her parti: kabul → kapanış → fiyat bulma → uzlaşma.

| Uç | Açıklama |
|---|---|
| `GET /v1/batch/current` | Parti numarası, kalan süre, çift listesi, işlem ücreti bandı. |
| `POST /v1/orders` | Emir gönder: `{pair, side, qty, limit_price, batch, client_id, expires_batch}`. Gövde eşik şifreli gönderilebilir (komitenin açık anahtarıyla), parti kapanana kadar içeriği kimse göremez. Yanıt: `order_id` + imzalı makbuz. |
| `DELETE /v1/orders/{id}` | Parti kapanmadan iptal. Kapanıştan sonra reddedilir (`409 batch_closed`). |
| `GET /v1/orders/{id}` | Durum: `queued`, `filled(qty, price)`, `partial`, `expired`, `rejected(reason)`. |
| `GET /v1/batch/{n}/result` | Uzlaşma fiyatı, hacim, doldurma özeti, zincir üstü taahhüt (hash). |
| `GET /v1/balances` | Kasa bakiyesi. (Gizli bakiye ilk sürümde yok; açık defter.) |
| `POST /v1/vault/deposit`, `POST /v1/vault/withdraw` | Kasaya para yatırma ve çekme talimatı. Çekme kuralı sözleşmede sabittir. |

Kurallar:
- `client_id` aynı cüzdan için tekil olmalı: aynı istek tekrar gelirse aynı yanıt döner (idempotent).
- Tüm emirler partinin tek uzlaşma fiyatından dolar. Limit fiyatı kötü ise doldurulmaz.
- Parti penceresi sabit uzunlukta (ör. 1–10 sn, parametre). Ajan için sabit gecikme öngörülebilirdir.

## 4. Ajan limitleri (sözleşmede)
Cüzdan sahibi, ajan anahtarı için bir "politika" kaydeder (sahip anahtarıyla, ajan değiştiremez):

```
policy = {
  agent_key, max_order_value, max_daily_value, allowed_pairs[], max_open_orders,
  expires_at, kill_switch (sahip anahtarı tek imzayla dondurur)
}
```
- `POST /v1/policy` (sahip imzası) / `GET /v1/policy/{agent_key}`.
- Limit aşan emir: `403 policy_violation` (hangi alan, hangi değer).
- Politika yoksa varsayılan düşük limit uygulanır (yeni adres sınırları, bölüm 5).
- Kimlik kanıtı eklenmişse limit çarpanı denetim ajanı kurallarıyla belirlenir; çarpan herkese açık yayımlanır.

## 5. Uyum ve denetim
- Yeni adresler ilk günlerde düşük limitle başlar (denetim ajanı, ayrı belge).
- Yaptırım listesinde eşleşen cüzdan: `451 sanctioned_address`, işlem alınmaz, kayıt açık günlüğe düşer.
- Denetim ajanı yalnızca geçici gecikme/limit düşürme uygulayabilir; kalıcı engel için komite onayı gerekir. Etkilenen cüzdan `GET /v1/flags/{wallet}` ile durumunu ve itiraz yolunu görür.
- İşlem günlüğü herkese açık: parti sonuçları ve makbuz özetleri zincirdedir.

## 6. Hatalar ve sınırlar
`400 bad_request`, `401 bad_signature`, `402 payment_required`, `403 policy_violation`, `409 nonce_reused | batch_closed | duplicate_client_id`, `429 rate_limited` (başlıkta `Retry-After`), `451 sanctioned_address`, `503 batch_unavailable`.
Hız sınırı: cüzdan başına saniyede N istek (parametre), aşımı x402 ile esnetilebilir.

## 7. Örnek akış (ajan)
1. `GET /v1/batch/current` → parti 4821, 3 sn kaldı.
2. `POST /v1/orders` (imzalı, x402 ödemeli) → `order_id`, makbuz.
3. Parti kapanır, uzlaşma çalışır.
4. `GET /v1/orders/{id}` → `filled`, fiyat = uzlaşma fiyatı.
5. Makbuz özeti, parti taahhüdü ile zincirde doğrulanır.

## 8. Açık maddeler (karar gerekli)
- Parti süresi ve minimum emir boyutu.
- Eşik şifrelemenin komite düzeni (kaç imza, gecikme maliyeti).
- x402 fiyatları ve hangi stablecoin(ler).
- Kimlik kanıtı biçimi (hangi standart kabul edilir) ve limit çarpanı.
- İlk işlem çifti: SOL / stablecoin riski (stablecoin dondurma) konusunda tercih.
- Test: API henüz yok. Önce sözleşme devnet testi, sonra bu şartname için sahte istemci ile entegrasyon testi yazılmalı.
