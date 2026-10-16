# Audit: Integration Swift cluster

Audit method-by-method tiga file klaster integrasi Swift. `Start`/`End` = nomor
baris di file masing-masing. Baseline commit `02c910932`.

File: `GitHubReleaseManager.swift` (737 baris), `SecureBookmarkManager.swift` (664),
`VaultOIDCHandler.swift` (618).

---

## File 1 — `GitHubReleaseManager.swift`

- Peran: singleton yang mengambil release GitHub ber-fase, memutuskan apakah build
  lebih baru ditawarkan, dan mengunduh/ekstrak zip update ke `~/Downloads`.

### Tipe & konstanta
`GitHubReleaseManager` (`@objc final class`, 13–737); `Config` (nested struct, 37–43). Tak ada `extension`.

Konstanta: `NSModalResponseView=1001` (15), `NSModalResponseDownload=1002` (16),
`githubURLStr = https://api.github.com/repos/%@/%@/releases` (18), `Log` (34), host reachability `"www.google.com"` (35),
Config default (40–42), `per_page` = `SAGitHubReleasePagination.pageSize = 100` (`GitHub.swift:424`),
`maximumPageCount = 10` (`GitHub.swift:425`), HTTP timeout 60 (141),
settling interval `15*60` (`GitHub.swift:85`), rollout knob `installationSeedPreferenceKey` (`GitHub.swift:272`, dibaca 263),
`clock_gettime_nsec_np(CLOCK_MONOTONIC)` (717), temp download `temporaryDirectory/"SequelAceUpdate"` (482),
extract `temporaryDirectory/UUID()` (563), `/usr/bin/unzip` `["-o", filePath, "-d", tempDir]` (589–590),
install `.downloadsDirectory` (631), pref `SPShowUpdateAvailable` (72) & `SPSkipNewReleaseAvailable` (367/409).

### `// MARK:`
`714 (Timing functions), 724 (ProgressViewControllerDelegate)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 15–36 | | konstanta + state (`sharedInstance`, `user`, `project`, `includeDraft`, `includePrerelease`, `appVariant`, progress VC/window, `download`, `currentReleaseName`, `availableReleaseName`, `currentRelease`, `availableRelease`, `releases`, `settlingRetry`, `checkTracker`, `Log`, `manager`) | State |
| 37 | 43 | `Config` (+ semua field) | Payload setup |
| 45 | 45 | `config` | Config static, divalidasi di init |
| 47 | 49 | `setup(_:)` | Simpan config sebelum konstruksi singleton |
| 51 | 67 | `init()` | Validasi config, salin ke state instance |
| 69 | 118 | `checkRelease(name:installedReleaseTag:isUserInitiated:)` | Gate policy, reset state, mulai fetch ber-halaman |
| 120 | 320 | `requestReleasePage(at:accumulatedReleases:pagesFetched:visitedPageURLs:checkID:installedBuildName:installedReleaseTag:isUserInitiated:)` | Fetch, filter, paginasi release + gate rollout |
| 322 | 341 | `scheduleSettlingRetry(after:installedBuildName:installedReleaseTag:isUserInitiated:)` | Jadwalkan re-cek tertunda di main queue |
| 343 | 353 | `displayRequestFailureIfNeeded(error:statusCode:isUserInitiated:)` | Alert kegagalan hanya bila actionable |
| 355 | 427 | `displayNewReleaseAvailableAlert(isUserInitiated:)` | Sheet alert dengan View/Download/Cancel |
| 429 | 712 | `downloadNewRelease(asset:)` | Unduh zip, unzip, pindah app ke Downloads |
| 716–722 | | `_monotonicTime()` / `_timeIntervalSinceMonotonicTime(comparisonTime:)` | Jam monotonic |
| 725–736 | | `cancelPressed()` / `closePressed()` | Delegate: cancel/teardown progress UI |

### Temuan / UNKNOWN
- `checkRelease` satu-satunya entry publik (69); terapkan `SAGitHubReleaseCheckPolicy.shouldCheck` (70–77) — build MAS + automatic-check-disabled exit dini (74–77). `checkTracker.begin` (84) + cabang "keep active user-initiated" (84–87) → cek otomatis tak bisa menimpa cek user.
- Paginasi = rantai `AF.request` rekursif (140) digerbangi `checkTracker.isCurrent` (146) + `defer` finish saat `shouldFinishCheck` tetap true (150–155); rekursi set `shouldFinishCheck = false` (189) → hanya halaman terminal finish. `visitedPageURLs` (113, 193) guard loop + `maximumPageCount`.
- Dua gate settling independen, sebelum (231) dan setelah (246) filter "no compatible zip", keduanya ke `scheduleSettlingRetry` (236, 251). Retry re-fetch penuh halaman 1..n; work item sebelumnya hanya dibatalkan di `checkRelease` (88–89). **UNKNOWN**: tak ada plafon jumlah retry di file ini — terminasi bergantung `SAGitHubReleaseSettlingPolicy.retryDelay` nil (`GitHub.swift:365`).
- Rollout: `rolloutSeed` sengaja `nil` untuk cek user-initiated, di-seed dari pref selain itu (261–263), lalu `SAGitHubReleaseRolloutPolicy.newestOfferedRelease` (265–270, policy `GitHub.swift:271`, `dailyPercentages=[1,2,5,10,20,50,100]`, `cohortBucketCount=10_000`). Efek: "Check for Updates" manual bypass rollout, cek startup menaatinya. `releases` lalu dikolaps ke ≤1 (271), keputusan perbandingan `>` (289).
- Download/install: alert `displayNewReleaseAvailableAlert` return `Bool` tapi caller buang (`_ =` 292); suppression ditulis saat dismiss (407–410) by `availableReleaseName` (367). Destinasi temp fix + `[.removePreviousFile, .createIntermediateDirectories]` (480–486). Unzip pakai `Process` stdout/stderr satu `Pipe` + loop drain sinkron **sebelum** `waitUntilExit` (592–613). `.app` dipindah ke `~/Downloads` (631–640), lalu prompt modal menawarkan `NSApp.terminate(nil)` (650–666).
- ETA = ekstrapolasi satu titik: `percentLeft = (1 - progress.fractionCompleted) + 1` (517), `ETA = diff * percentLeft` (520); subtitle update hanya saat estimasi turun (524–527). **UNKNOWN** apakah `+ 1` disengaja.
- **UNKNOWN**: **tak ada verifikasi integritas** arsip/app yang diunduh di 429–712 (tanpa checksum/signature/notarization).
- **Dead/missing wiring**: `manager?.isReachable` hanya dibaca (312, 706) tapi tak ada `startListening()` di workspace dan tipe tak vendored in-tree — cabang diagnostik kemungkinan tak pernah jalan. `// MARK: ProgressViewControllerDelegate` (724) menamai protokol yang tak dideklarasikan di workspace; dua method (725, 732) adalah instance method biasa. `error.isExplicitlyCancelledError` (702) tak dideklarasikan di file tracked — **UNKNOWN**.

---

## File 2 — `SecureBookmarkManager.swift`

- Peran: memiliki security-scoped bookmark persisten untuk pane file-preference/
  known_hosts: migrasi data legacy, re-resolve + refresh bookmark stale, dan
  melacak URL yang sedang memegang security-scoped access.

### Tipe & konstanta
`SecureBookmarkManager` (`@objc final class`, 21–664). Tak ada nested type/extension. Header komentar 12–19 mendaftar permukaan publik.

`URLBookmarkResolutionWithSecurityScope = 1<<10` (29), `URLBookmarkCreationWithSecurityScope = 1<<11` (30),
`Log` (37), `prefs` (32), prefix volume `"/Volumes/"` (231), arity root volume `prefix(3)` (240),
pref key `SPStaleSecureBookmarks` (64), `SPKnownHostsBookmarks` (65), `SASecureBookmarks` (67),
`SPSecureBookmarksHaveBeenMigrated` (76), `SPSecureBookmarksOldFormat` (155), notifikasi `SPBookmarksChanged` (60, 622, 640).

### `// MARK:`
**Tidak ada** (grep 0). Navigasi hanya via doc komentar `///` (94, 161, 362, 477, 546) + header 12–19.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 22–37 | | `sharedInstance`, `bookmarks`, `staleBookmarks`, `resolvedBookmarks`, `knownHostsBookmarks`, opsi bookmark, `prefs`, `observer`, `iChangedTheBookmarks`, `bookmarksHaveBeenMigrated`, `Log` | State |
| 40 | 92 | `init()` | Muat pref, pasang observer, migrasi, re-request |
| 101 | 159 | `migrateBookmarks()` | Re-encode + persist bookmark legacy (sekali) |
| 166 | 228 | `reRequestSecureAccessToBookmarks()` | Resolve semua bookmark, refresh stale, persist |
| 230 | 242 | `unavailableMountedVolumeRoot(for:)` | Root `/Volumes/...` saat volume absen |
| 244 | 265 | `keepResolvedBookmark(_:originalKey:bookmarkData:)` | Simpan bookmark, mulai scope, bersihkan referensi stale |
| 267 | 296 | `refreshedEncodedBookmarkData(for:storedBookmark:)` | Re-create bookmark dengan opsi tersimpan + re-encode |
| 298 | 313 | `removeStaleBookmarksCoveredByResolvedDirectoryBookmarks()` | Buang stale child dari direktori ter-resolve |
| 315 | 322 | `resolvedDirectoryPath(for:)` | Path direktori ternormalisasi |
| 324–331 | | `normalizedFilePath(forBookmarkPath:)` / `normalizedDirectoryPath(_:)` | Delegasi normalisasi path |
| 333–352 | | `removeStaleBookmarkReferences(for:)` / `hasStaleBookmarkReference(for:)` | Hapus/uji referensi stale |
| 354 | 360 | `deduplicateStaleBookmarks()` | Dedup path stale by bentuk ternormalisasi |
| 368 | 475 | `addBookmarkFor(url:options:isForStaleBookmark:isForKnownHostsFile:)` | Buat/refresh bookmark untuk URL + persist |
| 481 | 544 | `bookmarkFor(filename:)` | Resolve bookmark tersimpan ke URL, refresh stale |
| 550 | 632 | `revokeBookmark(filename:)` | Hapus entri known-hosts, stale, standar |
| 634–648 | | `addStaleBookmark(filename:)` / `addKnownHostsBookmark(filename:)` | Append + persist + notifikasi |
| 651 | 659 | `stopAllSecurityScopedAccess()` | Stop scope untuk semua URL ter-resolve |
| 661 | 663 | `deinit` | Invalidate observer KVO UserDefaults |

### Temuan / UNKNOWN
- `init` pasang observer KVO pada key pref (49) + force-unwrap `change.newValue!` saat perubahan bukan dari sini (51–54) — jalur crash bila key dihapus eksternal. **UNKNOWN** apakah caller bisa mengosongkan `SASecureBookmarks` di luar kelas.
- `bookmarks`/`staleBookmarks`/`knownHostsBookmarks` adalah `@objc var` eksternal (24–25, 28) → write dari ObjC melewati `iChangedTheBookmarks` (35) + write-back pref.
- Migrasi (101–159) selalu coba opsi security-scope sebagai tebakan (komentar 126), append kegagalan ke `staleBookmarks`, backup nilai lama di `SPSecureBookmarksOldFormat` (155), baru re-request (80–83). Bila key pref kosong, `init` return sebelum migrasi (67–70) → `SPSecureBookmarksHaveBeenMigrated` tak di-set.
- Re-resolve (166–228) mulai dari salinan (176), bersihkan kedua array (179–180); entri stale di-refresh+keep (202–203) atau ditandai stale (206). Resolusi hanya pakai opsi resolution (194), rasional di 161–165.
- `unavailableMountedVolumeRoot` mengasumsikan path security-scoped selalu `/Volumes/<name>/...` + `prefix(3)` komponen standardized (231–241). **UNKNOWN**: tanpa penanganan network mount yang path lokalnya bukan `/Volumes`, dan prefix symlink bisa non-root.
- `addBookmarkFor` punya dua exit sukses, 416 (entry existing di-refresh/reuse) dan 469 (baru dibuat); cabang `isForStaleBookmark == true` menghapus dict existing (418–424) + komentar `// JCS - Not sure we'll ever get here` (419). Matching terhadap bentuk encoded **dan** percent-decoded (371–374, force-unwrap `removingPercentEncoding!` 373).
- `revokeBookmark` hapus known-hosts + stale by perbandingan sanitized (558–574) dan entri standar by resolve tiap dict (578–619). **Tak pernah** panggil `stopAccessingSecurityScopedResource()`; komentar 597–599 + `FIXME: ACTUALLY ... DO WE NEED ALL THIS?` menandai pertanyaan; kode hanya drop URL dari `resolvedBookmarks` (601). **UNKNOWN**: scope sengaja di-leak atau ditangani tempat lain.
- `stopAllSecurityScopedAccess` memutasi `resolvedBookmarks` saat iterasi (655–658) — aman via value semantics tapi O(n²); satu-satunya tempat panggil `stopAccessingSecurityScopedResource()`.
- **UNKNOWN**: observer hanya di-invalidate di `deinit` (662) dan `sharedInstance` (22) tak pernah dilepas → observer praktis seumur proses.
- `normalizedFilePath(forBookmarkPath:)` (324–326) delegasi murni ke `SABookmarkPathNormalizer` (`NSAlertExtension.swift:11`).

---

## File 3 — `VaultOIDCHandler.swift`

- Peran: menjalankan alur login Vault OIDC berbasis browser — bind listener HTTP
  loopback, buka auth URL, tangkap callback, tukar code jadi token, dan simpan di
  Keychain per server+mount.

### Tipe & konstanta
`VaultOIDCError` (enum Error/LocalizedError, 17–41); `VaultOIDCHandler` (43–604, nested `private struct ActiveLogin` 51–54); `extension VaultOIDCError: Equatable` (606–618).

`log` (45), `callbackTimeoutSeconds = 120` (46), `keychainService = "Sequel Ace Vault Token"` (47),
`callbackPort = 8250` (247; cocok vault-plugin-auth-jwt CLIHandler, komentar 244–246),
redirect URI `http://localhost:8250/oidc/callback` (275), path check `GET /oidc/callback` (517),
binding `requiredInterfaceType = .loopback` (485), sintesis `EADDRINUSE` (503–507),
respons HTTP 200 `text/html` `Connection: close` (521–522), limit baca request 4096/8192 (554, 567),
token random 16 byte via `SecRandomCopyBytes` base64url tanpa padding (579–588),
scope key `"<baseURL.absoluteString>|oidc_mount=<mount>"` (430), mount default `"oidc"` (435).

### `// MARK:`
`49 (Cancellation), 190 (Per-mount token scoping), 359 (Token persistence), 463 (ObjC bridge), 476 (Callback listener)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 18–23 | | `VaultOIDCError` cases | cancelled/noAvailablePort/loginInProgress/callbackTimeout/malformedCallback/vaultError |
| 25 | 40 | `errorDescription` | Pesan terlokalisasi per case |
| 45–47 | | `log`/`callbackTimeoutSeconds`/`keychainService` | Konstanta |
| 51–54 | | `ActiveLogin` (+ `semaphore`/`cancelled`) | State login per-identifier |
| 56–65 | | `loginLock`/`activeLogins`/`loginInProgress` | State eksklusivitas |
| 69–82 | | `beginExclusiveLogin()`/`endExclusiveLogin()` | Klaim/lepas slot login tunggal |
| 87–113 | | `prepareActiveLogin()`/`clearPreparedActiveLogin(identifier:)`/`clearPreparedActiveLogin()` | Registrasi pra-login + cleanup |
| 118–144 | | `cancelActiveLogin(identifier:)`/`cancelActiveLogin()` | Tandai permanent cancel + signal |
| 146–160 | | `registerActiveLogin`/`clearActiveLogin` | Attach/lepas semaphore |
| 162–188 | | Test hooks + `isActiveLoginCancelled(identifier:)` | Hook test + query cancel |
| 194–242 | | `hostTokenLock`/`tokenByScope`/`persistedTokenLock`/`persistedTokenByScopeForTesting` + `cachedToken`/`storeToken`/test hooks | Cache token |
| 247 | 247 | `callbackPort` | Port callback loopback 8250 |
| 252 | 357 | `login(baseURL:mount:identifier:)` | Alur penuh: listen, auth URL, tunggu, tukar, persist |
| 361 | 392 | `saveToken(_:for:mount:)` | Update lalu add item Keychain untuk scope |
| 394 | 423 | `readPersistedToken(for:mount:)` | Baca + validasi token Keychain |
| 425–444 | | `keychainAccount(for:mount:)`/`tokenScopeKey`/`normalizedMount`/`keychainQuery(account:)` | Key/query Keychain |
| 446 | 461 | `keychainTokenExistsForTesting(baseURL:mount:)` | Hook test keberadaan item |
| 467 | 474 | `loginObjC(baseURL:mount:error:)` | Bridge ObjC → token/NSError |
| 478 | 546 | `startCallbackListener(onCallback:)` | Bind listener loopback, HTTP callback one-shot |
| 548 | 574 | `receiveHTTPRequest(on:buffer:completion:)` | Buffer byte sampai baris request pertama |
| 578 | 589 | `randomBase64URLToken()` | Token base64url 16 byte CSPRNG |
| 591 | 603 | `parseQueryParams(from:)` | Ekstrak query param dari request line |
| 607 | 617 | `VaultOIDCError.==` | Equatable antar case error |

### Temuan / UNKNOWN
- Eksklusivitas diklaim sebelum state apa pun dibuat (257–258, rasional 253–256). Port fix 8250 (247) tanpa fallback: listener asing memegang port → `.noAvailablePort` dari `listenerError` (540–543) atau state `.waiting` (503–507).
- Listener dibuat 266 dan semaphore diregistrasi 271 **sebelum** panggilan blocking `VaultClient.oidcAuthURL` (285) justru agar cancel bisa menginterupsi fase setup (271–273, komentar 302–303). Checkpoint cancel di 273 dan 292; buka browser dicek ulang di main queue sebelum `NSWorkspace.open` (295–299).
- `login` memblok thread pemanggil di `callbackSemaphore.wait(timeout: .now()+120)` (305); timeout → `callbackTimeout` (306), params nil → `cancelled` (307–308). Caller `VaultAuthManager.swift:310/435`; prep/cancel dari `SAConnectionWindowController.swift:412/284` dan `SAVaultRoleListController.swift:180/149`.
- **Validasi state sengaja dimatikan**: `guard callbackState == state` dikomentari dengan justifikasi panjang (317–330) — vault-plugin-auth-jwt mengganti state opaque sendiri, CSRF didelegasikan ke cache state server-side via `oidcCallback` (336–337). `state`/`nonce` klien (279–280) tetap dikirim ke `oidcAuthURL` (287) dan nonce di-replay (337). **UNKNOWN**: tak ada verifikasi ulang bahwa token kembali milik mount yang diminta selain scope keyed 362/430.
- Token dipersist + cache (350–351) **sebelum** app re-activate (354); komentar 344–349 mencatat cancel pasca-tukar meninggalkan token Keychain live yang sengaja dipakai ulang.
- Robustness callback: `callbackFired` one-shot di bawah `callbackLock` (491–494, 525–529); HTTP 200 dikirim **sebelum** `parseQueryParams` (523–532). **UNKNOWN**: bila `.contentProcessed` `connection.send` tak pernah fire, `onCallback` dilewati senyap dan login menunggu 120 s penuh.
- Path check pakai `hasPrefix("GET /oidc/callback")` (517) — juga menerima `/oidc/callback-extra`; **UNKNOWN** apakah sengaja longgar.
- Persistensi: dua-tier (map in-process → Keychain, read-through re-cache 201–211). Scope key `baseURL + "|oidc_mount=" + mount` (429–436). `saveToken` update, fallback add hanya saat `errSecItemNotFound` (374–391); kedua jalur gagal hanya `os_log` (380, 390).
- **UNKNOWN**: query (438–444) tak set `kSecAttrAccessible` → item pakai default platform; tanpa hardening "when unlocked/this device only".
- State cancel bisa hidup lebih lama dari login: `cancelActiveLogin(identifier:)` pada identifier tak dikenal **membuat** entry `cancelled: true` tanpa semaphore (129–131); hanya dibersihkan `clearPreparedActiveLogin(identifier:)` (98–107). **UNKNOWN**: tak ada batas pertumbuhan map.
- `cancelActiveLogin()` (136–144) signal semua semaphore live tapi tak membersihkan apa pun; cleanup bergantung `defer clearActiveLogin` tiap `login` (272) yang match by identitas semaphore (156).
- `Equatable` menganggap **apa pun** dua `vaultError` sama (614) — kasar untuk assertion test.
- Entry test-only adalah static non-private di kode produksi (162–188, 220–242, 446–461), dipakai `UnitTests/VaultOIDCHandlerTests.swift:27–205`. **UNKNOWN**: `loginObjC` (467–474) dianotasi ObjC (466) tapi tak ada call site ObjC ditemukan.

---

## Ringkasan lintas-file
- Ketiganya `@objc`/`@objcMembers` `NSObject` singleton atau handler static-only; hanya `VaultOIDCHandler` punya `// MARK:` API publik (5). `SecureBookmarkManager` **nol** MARK (gap navigabilitas utama di klaster ini).
- Arah dependensi keluar saja: `GitHubReleaseManager` → `GitHub.swift`; `SecureBookmarkManager` → `SecureBookmark.swift:12`/`SecureBookmarkData.swift:11`/`SABookmarkPathNormalizer`; `VaultOIDCHandler` → `VaultClient.swift:175/202`.
- Item UNKNOWN bersama: cabang `manager?.isReachable` (`GitHubReleaseManager.swift:312, 706`, tanpa `startListening`), `ProgressViewControllerDelegate` tak terdefinisi (724) + `isExplicitlyCancelledError` (702), tak ada cek integritas pasca-unduh (429–712), scope bookmark yang direvoke bocor (`SecureBookmarkManager.swift:597–601`), dan perbandingan state OIDC sengaja dimatikan (`VaultOIDCHandler.swift:317–330`).
