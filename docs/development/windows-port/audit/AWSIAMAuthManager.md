# Audit: `AWSIAMAuthManager.swift`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Other/AWS/AWSIAMAuthManager.swift`. Baseline commit `02c910932`.

- File: 967 baris.
- Peran: facade statis yang mengubah profile AWS CLI menjadi token auth RDS IAM —
  resolusi kredensial profile→SSO/console/role-assume (dengan cache in-memory),
  penyimpanan secret di keychain, fetch katalog region, dan combo box pemilih
  region AppKit.
- Total: 71 definisi (6 tipe).

## 1. Tipe & konstanta

| Tipe | Jenis | Baris |
| --- | --- | --- |
| `AWSIAMAuthError` | `@objc enum : Int, Error, LocalizedError` | 36–60 |
| `AWSIAMAuthManager` | `@objcMembers final class` | 63–641 |
| `AWSIAMAuthManager` (extension) | `extension` | 645–763 |
| `SAAWSRegionComboBoxPreparationDelegate` | `@objc protocol` | 768–771 |
| `SAAWSRegionComboBox` | `@objc final class : NSComboBox` | 775–956 |
| `SAAWSRegionComboBoxCell` | `@objc final class : NSComboBoxCell` | 959–967 |

Konstanta/cache: `keychainServicePrefix="Sequel Ace AWS"` (67), `log` (68),
`awsRegionCatalogURL = https://ip-ranges.amazonaws.com/ip-ranges.json` (69–74, `preconditionFailure` bila literal salah),
`awsRegionCacheKey`/`...TimestampKey` (75–76), **`awsRegionCacheTTL = 7 hari`** (77),
`fallbackRegions` (78–113, **34 region**), `credentialCache` (119), `cacheLock` (120),
`default/minimum/maximumRoleCredentialCacheDuration` (121–123: 3600/1/3600),
`cacheExpirationMargin = 300` (126). Literal tak dijadikan konstanta: `"us-east-1"` (168),
`"default"` (174), `durationSeconds: 3600` (416), `timeoutInterval: 15` (549).

## 2. Semua `// MARK:`

`65 (Keychain Constants), 115 (Credential Caching), 133 (Token Generation), 198 (Credential Loading), 292 (Credential Caching — **duplikat** heading #2), 430 (Keychain Management), 514 (Utility Methods), 590 (Region Catalog Helpers), 643 (Objective-C Compatibility), 765 (Region Picker)`.

## 3. Tabel definisi (urut baris, per tipe)

### `AWSIAMAuthError` (36–60)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 44 | 59 | `errorDescription` | Map tiap case error ke string tampil terlokalisasi |

### `AWSIAMAuthManager` — konstanta & state (63–641)
Konstanta 67–77 (`keychainServicePrefix`, `log`, `awsRegionCatalogURL`, key cache, TTL);
`fallbackRegions` (78–113); `credentialCache` (119); `cacheLock` (120);
`defaultRoleCredentialCacheDuration` (121); `minimumRoleCredentialCacheDuration` (122);
`maximumRoleCredentialCacheDuration` (123); `cacheExpirationMargin` (126).

### Token generation & credential loading (128–428)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 128 | 131 | `preferredSTSRegion(baseRegion:fallbackRegion:)` | Pilih region dasar ter-trim, else fallback |
| 147 | 196 | `generateAuthToken(...)` | Resolve region/profile + bangun token RDS IAM |
| 201 | 268 | `loadCredentialsFromProfile(_:region:parentWindow:shouldContinue:)` | Rantai dispatch: SSO → console login → role assumption |
| 271 | 279 | `resolveLoginCredentials(_:profileName:)` | Resolve + cache kredensial console `aws login` |
| 282 | 290 | `resolveSSOCredentials(_:profileName:)` | Resolve + cache kredensial IAM Identity Center |
| 295 | 312 | `getCachedCredentials(for:)` | Kembalikan cache belum kedaluwarsa, evict yang kedaluwarsa |
| 315 | 322 | `cacheCredentials(_:for:duration:)` | Simpan kredensial + tanggal kedaluwarsa |
| 324 | 338 | `cacheDurationForAssumedCredentials(_:)` | Clamp lifetime 1–3600 s |
| 341 | 350 | `clearCachedCredentials(for:)` | Buang satu entri cache profile, atau semua bila nil |
| 353 | 395 | `assumeRoleWithMFA(baseCredentials:profileName:region:parentWindow:shouldContinue:)` | Prompt MFA lalu STS AssumeRole dengan MFA |
| 398 | 428 | `assumeRoleWithoutMFA(baseCredentials:region:)` | STS AssumeRole 1 jam tanpa MFA |

### Keychain, utility, region catalog (430–640)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 433 | 466 | `saveSecretKey(_:forAccessKey:connectionName:)` | Hapus lalu re-add item generic-password keychain |
| 469 | 494 | `getSecretKey(forAccessKey:connectionName:)` | Baca data secret key dari item keychain |
| 497 | 512 | `deleteSecretKey(forAccessKey:connectionName:)` | Hapus item keychain, toleran not-found |
| 517–524 | | `credentialsFileExists`, `availableProfiles()` | Passthrough ke `AWSCredentials` |
| 528 | 533 | `cachedOrFallbackRegions()` | Kembalikan region ter-cache, else fallback |
| 538 | 578 | `refreshAWSRegionsIfNeeded(completion:)` | Refresh katalog region bila stale, else sajikan cache |
| 581–588 | | `isRDSHostname(_:)`, `regionFromHostname(_:)` | Passthrough |
| 592 | 610 | `regionsFromIPRangesResponse(_:)` | Parse JSON `prefixes`/`ipv6_prefixes` → region valid tersortir |
| 612 | 619 | `mergeWithFallbackRegions(_:)` | Union region ter-parse + fallback, tersortir |
| 621 | 627 | `cachedRegions()` | Baca array region non-kosong dari UserDefaults |
| 629 | 635 | `isRegionCacheFresh()` | Banding timestamp tersimpan vs TTL 7 hari |
| 637 | 640 | `persistRegionCache(_:)` | Tulis array region + timestamp ke UserDefaults |

### `extension AWSIAMAuthManager` — Objective-C compatibility (645–763)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 649 | 679 | `generateAuthTokenInBackground(...shouldContinue:completion:)` | Resolve background + cek cancellation, completion main-thread |
| 684 | 710 | `generateAuthTokenObjC(...error:)` | Varian pointer NSError, nil saat gagal |
| 716 | 750 | `generateAuthTokenInBackground(...completion:)` | Resolve background + completion main-thread tanpa hook cancellation |
| 753 | 762 | `presentableError(_:profile:)` | Bungkus error jadi NSError, delegasi hint sign-in |

### `SAAWSRegionComboBoxPreparationDelegate` (768–771)
`prepareAWSRegionComboBox(_:completion:)` (769–770).

### `SAAWSRegionComboBox` (775–956)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 776–790 | | `preparationDelegate`, `hasPreparedPopup`, `shouldOpenPopupAfterPreparation`, `isPreparingPopup`, `interveningInputMonitor`, `preparationIndicator` | State + indikator progress |
| 792 | 797 | `mouseDown(with:)` | Tunda klik ke preparation atau fallback `super` |
| 799 | 804 | `accessibilityPerformShowMenu()` | Aksi menu aksesibilitas lewat preparation |
| 808 | 836 | `preparePopupIfNeeded()` | Mulai preparation first-use, lapor wait/no-wait |
| 838 | 857 | `layout()` | Posisi indikator menghormati layout direction |
| 860 | 862 | `cancelPendingPopupOpening()` | Bersihkan flag pending-open setelah input intervening |
| 864 | 873 | `shouldPreparePopup(for:)` | Deteksi klik area cell yang trackable non-editable |
| 875 | 882 | `monitorForInterveningInput()` | Pasang monitor lokal membatalkan pending popup |
| 884 | 891 | `stopMonitoringInterveningInput()` | Hapus monitor lokal bila terpasang |
| 893 | 951 | `finishPreparingPopup()` | Akhiri preparation, opsional sintesis event mouse open |
| 953 | 955 | `deinit` | Hapus event monitor saat dealloc |

### `SAAWSRegionComboBoxCell` (959–967)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 960 | 966 | `accessibilityPerformShowMenu()` | Rutekan aksi menu cell ke preparation combo box |

## 4. Temuan menonjol / UNKNOWN

**Presedensi kredensial (SSO > console login > role assume > static)** — urutan hard-coded di `loadCredentialsFromProfile`: SSO (218–220), console sign-in (224–226), role assumption (229–232), else base credential langsung (231). Predikat `resolvesThroughConsoleSignIn` di luar file = `!isSSOProfile && isLoginProfile && !isValid && !requiresRoleAssumption` (`AWSLoginCredentialsProvider.swift:95-97`). **UNKNOWN**: web-identity/process/`credential_process` `source_profile` chain dan `AWS_PROFILE` env tak jelas ditangani di sini. SSO branch jalan **sebelum** cek role-arn → SSO profile **dengan** `roleArn` akan skip assume-role lokal (218, 287).

**STS & region** — `preferredSTSRegion` (128–131, dipanggil 376/407) → region profile menang. MFA butuh `roleArn` + `mfaSerial` (360–363), cancel dialog → `mfaCancelled` (366–373). No-MFA selalu `durationSeconds: 3600` (416) + `roleSessionName: nil` (413). Kedua jalur mengubah `NSError` apa pun jadi `roleAssumptionFailed` (388–391, 421–424), membuang kode asli.

**Caching** — cache static proses-lokal tak terbatas per jumlah profile (119); eviksi hanya via cek expiry (305–309) atau clear (341–350). `clearCachedCredentials` publik tapi **tak ada call site di file ini**. Key hanya **nama profile** (119, 239) — **risiko**: dua koneksi profile sama beda region/role session berbagi satu entri. `cacheDurationForAssumedCredentials` clamp `min(3600, max(1, secondsRemaining))` (334–337). `defaultRoleCredentialCacheDuration` == `maximumRoleCredentialCacheDuration` (121, 123) = 3600.

**Fetch region** — katalog adalah **prefix list IPv4/IPv6**, bukan API enumerasi region resmi (70); region diturunkan dari entri prefix (597–601) + filter `!= "global"` + `RDSIAMAuthentication.isValidAWSRegion` (603–605). Fetch `.reloadIgnoringLocalCacheData` 15 s (548–549); gagal → `cachedOrFallbackRegions()` di main queue (555–557, 565–567). `completion` **selalu main queue** termasuk fast path (538–543, 574–576). Freshness hanya timestamp, **tanpa ETag** (629–635, 546–550). `cachedRegions()` re-merge fallback tiap baca (626) → data tersimpan tak pernah kurang dari 34 entri.

**Error enum** — 6 case raw `Int` (37–42); `presentableError` map ke NSError domain `"AWSIAMAuthErrorDomain"` code = `rawValue` (753–759), else delegasi ke `SAAWSSignInCommand.presentableError` (761). **UNKNOWN**: hanya `credentialsNotFound` (213), `mfaCancelled` (372), `roleAssumptionFailed` (362, 391, 403, 424), `tokenGenerationFailed` (194) yang di-throw; `credentialsInvalid` (38) dan `keychainError` (42) **tak punya throw site di sini**.

**Lain**
- Parameter `accessKey`/`secretKey` manual diterima tapi **diabaikan ekplisit** (142–143, 172, 666, 736) — dead API.
- Dua overload `generateAuthTokenInBackground` beda hanya hook `shouldContinue` (649–679 vs 716–750).
- `shouldContinue` diinvoke via `DispatchQueue.main.sync` (660) dari background queue — hop sinkron ke main; **UNKNOWN** apakah caller bisa memanggil dari main (deadlock).
- Item keychain `kSecClassGenericPassword` service `"Sequel Ace AWS - <connectionName>"`, account = access key (438–439, 473–474), accessible `kSecAttrAccessibleWhenUnlocked` (455); `saveSecretKey` return false tanpa detail error (460–463).
- Komentar 397 & 228 menandai "FIX" historis.
- `SAAWSRegionComboBox.finishPreparingPopup` menyintesis pasangan mouse-down/up nyata untuk membuka popup (924–950), digerbangi cek key/main-active/visibility/enabled (905–912). **UNKNOWN**: `mouseDown` (792–797) return tanpa `super` saat preparation mulai dan tak retrigger sendiri — popup bergantung penuh pada sintesis event (949–950).
- `preparationIndicator` dibuat lazy + ditambah sebagai subview dalam closure init (783–790); `layout()` mengaksésnya tanpa syarat (851) → memaksa pembuatan saat layout pertama.
