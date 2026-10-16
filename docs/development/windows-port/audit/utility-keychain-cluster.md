# Audit: Utility / Keychain / Data / Debugging cluster

Audit method-by-method klaster utility. `Start`/`End` = nomor baris di file masing-masing.
Baseline commit `02c910932`.

File: `Utility/OSLog.swift` (521), `Utility/SPFunctions.m` (453), `Utility/SADragPasteboard.swift` (262),
`Utility/SAPrintUtility.swift` (220), `Extensions/NSAlertExtension.swift` (375),
`Keychain/SAKeychain.swift` (286), `Data/SPConstants.m` (535), `DebuggingSupport/SPLogger.m` (261),
`FileCompression/SPFileHandle.m` (484).

---

## 1. `OSLog.swift` (521)

Extension `OSLog` — logging berlevel (`log`/`info`/`debug`/`trace`/`error`/`fault`) dengan privacy per-call + capture source-location opsional.

### Tipe
`extension OSLog` (103) dengan `enum Options` (109–136: `defaultPrivacy` 112–118, `includeSourceLocationInValueLogs` 122–128, `logFileName` 132–134), `enum Privacy` (139–146: visible/redacted);
`protocol CustomLogRepresentable` (418–421); `struct LogMessage` (424–456: `format` 427, `args` 435); `extension CustomLogRepresentable` (458–507); `private struct AnyLoggable` (509–515); conformances NSError/String/Bool/Int/UInt (517–521).

### `// MARK:`
`105 (Nested Types), 148 (Default), 175 (Info), 216 (Debug), 270 (Error), 297 (Fault), 324 (Internal), 362 (CustomLogRepresentable)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 159–173 | | `log(format:args:)` / `log(_:privacy:...)` | Log level default |
| 186–214 | | `info(...)` ×2 | Log level info |
| 220 | 229 | `trace(file:function:line:)` | Log lokasi source call-site di level debug |
| 240–268 | | `debug(...)` ×2 | Log level debug |
| 281–295 | | `error(...)` ×2 | Log level error |
| 308–322 | | `fault(...)` ×2 | Log level fault |
| 327–357 | | `_etcetera_log(value:...)` / `_etcetera_log(representation:type:)` | Resolve lalu `os_log` (0–9 arg) |
| 438–454 | | `LogMessage.init(_:_:)` / `init(format:array:)` | Init variadik (assert ≤9) |
| 461–505 | | `logRepresentation(...)` ×3 / `logDescription` | Representasi default |
| 512–514 | | `AnyLoggable.init(_:)` | Bungkus nilai |

### Temuan
- `info`/`debug`/`trace` bercabang ke `OSLogType.default` di `targetEnvironment(simulator)` (workaround Xcode lama) (190–196, 206–212, 221–227, 244–250, 260–266).
- `_etcetera_log(representation:type:)` cap keras 9 arg (345–356); `LogMessage` assert `< 10` (439, 451).
- `logDescription` hanya hormati `CustomDebugStringConvertible` di `#if DEBUG` (496–500).
- Empat conformance kosong (517–520) mengandalkan default protocol extension.

---

## 2. `SPFunctions.m` (453)

Fungsi utilitas C — dispatch main/background, byte acak aman, heuristik SSH "no route to host" jaringan lokal, dan parsing URL `mysql://`.

### Tipe/konstanta
Tak ada kelas/ivar. Helper static: `SPValidMySQLConnectionURLTypes` (54), `SPTrimmedHostCandidate` (162), `SPIsPrivateIPv4Address` (177), `SPIsPrivateIPv6Address` (195), `SPAddSSHRegexMatches` (237).

### `#pragma mark`
**Tidak ada.**

### Fungsi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 37 | 52 | `SPValidMySQLConnectionURLQueryParameters` | Nama parameter query MySQL URL valid |
| 54–57 | | `SPValidMySQLConnectionURLTypes` (static) | Set string tipe koneksi valid |
| 59–92 | | `SPMainQSync` / `executeOnMainThreadAfterADelay` / `executeOnLowPrioQueueAfterADelay` | Dispatch + delay |
| 94–127 | | `SPMainLoopAsync` / `dispatch_once_on_main_thread` / `executeOnBackgroundThreadSync` / `executeOnBackgroundThread` | Dispatch |
| 129–160 | | `SPBetterRandomBytes` / `SPIntS2U` / `SPBoxNil` | Utils |
| 162–204 | | `SPTrimmedHostCandidate` / `SPIsPrivateIPv4Address` / `SPIsPrivateIPv6Address` | Helper host |
| 206 | 235 | `SPIsLikelyLocalNetworkHost` | Heuristik host lokal |
| 237–283 | | `SPAddSSHRegexMatches` / `SPSSHNoRouteToHostLikelyLocalNetworkPrivacyIssue` | Diagnosa SSH |
| 285 | 406 | `SPExtractConnectionDetailsFromMySQLURL` | Parse `mysql://` → dict detail |
| 408–453 | | `SP_swizzleInstanceMethod` / `DumpObjCMethods` / `intSortDesc` | Debug + sort |

### Temuan
- `SPMainQSync` (59–72) + `executeOnBackgroundThreadSync` (114–127) pakai `dispatch_queue_set_specific`/`dispatch_get_specific` untuk hindari deadlock re-entrant — dependensi semantik `SPMainQSync` (dipakai juga `SAKeychain.presentErrorSync`).
- `SPIsLikelyLocalNetworkHost` anggap hostname tanpa titik sebagai lokal (230–232) — **dieksplisitkan in-file** sebagai heuristik yang bisa salah klasifikasi alias SSH host publik (226–229).
- `SPExtractConnectionDetailsFromMySQLURL` return NO + isi `invalidParameters` pada key query tak dikenal (301–303, 341–344); `autoConnect` hanya bila ada password di URL (386).

---

## 3. `SADragPasteboard.swift` (262)

Plumbing pasteboard item + aturan "tolak drag ini" untuk table view yang dimigrasi dari `writeRowsWithIndexes:toPasteboard:`.

### Tipe/konstanta
`@objc final class SADragPasteboard: NSObject` (19), stateless (semua static). UTI: `tableRowType` (32), `sslCipherType` (35), `dragRowType` (42), `navigatorSchemaPathsType` (47), `navigatorTableDataType` (52).

### `// MARK:`
`21 (Types), 54 (Writing), 75 (Reading), 87 (Whole-drag payloads), 170 (Payload field resolution), 237 (Refusal rules)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 58–85 | | `item(string:forType:)` / `item(row:forType:)` / `strings(from:forType:)` | Write/read item |
| 115–168 | | `dragRowItem(row:)` / `payloadItem(on:)` / `attach(dragString:to:)` / `attach(string/propertyList/data:forType:to:)` | Payload whole-drag |
| 185–235 | | `schemaPath(fromKey:delimiter:)` / `carriesDragText(schemaPath:delimiter:)` / `columnName(forClickedColumn:identifiers:columnNames:)` | Resolusi field |
| 246–261 | | `refusesMultiRowDrag(row:selectedRows:)` / `refusesDrag(row:selectedRows:excludedRow:)` | Aturan tolak |

### Temuan
- Tipe pasteboard **harus UTI-conformant**; nama legacy yang ditolak `NSPasteboardItem` menghasilkan item kosong — didokumentasikan & di-assert (23–29, 64, 146, 156, 166).
- Desain "marker-only per-row items" **load-bearing** karena `.string` digabung antar item, sedangkan tipe lain hanya baca item pertama (100–110).
- `columnName(...)` sengaja terima `[Any]` (array ObjC dengan `NSNull`) + pakai `NSString.integerValue` untuk kelonggaran ObjC (216–220, 228–231).
- `refusesDrag` perlakukan `NSNotFound` sebagai "tak ada baris" (258).

---

## 4. `SAPrintUtility.swift` (220)

Bangun `NSPrintOperation` untuk `WKWebView` dengan page setup bersama + accessory print panel + renderer HTML offscreen.

### Tipe/konstanta
`let printBackgroundPreferenceKey = "PrintBackground"` (16, inline; sinkron dengan `SPPrintBackground` 13–15);
`@objc final class SAPrintUtility` (24, static);
`final class SAPrintAccessoryController: NSViewController, NSPrintPanelAccessorizing` (87) — `webView` weak (89), `printsBackgrounds` dynamic (92–97);
`@objc final class SAHTMLPrintRenderer: NSObject, WKNavigationDelegate` (156) — `webView` (158), `completion` (159).

### `// MARK:`
`132 (NSPrintPanelAccessorizing)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 30–80 | | `configuredPrintInfo()` / `printOperation(for:)` | Print info + operasi |
| 92–145 | | `printsBackgrounds` / init ×2 / `loadView()` / `toggled(_:)` / `localizedSummaryItems()` / `keyPathsForValuesAffectingPreview()` | Accessory panel |
| 164–203 | | `printHTMLString(_:completionHandler:)` / `invalidate()` / delegate ×3 | Renderer |
| 209–219 | | `webViewWebContentProcessDidTerminate(_:)` / `finish(with:from:)` | Guard + completion sekali |

### Temuan
- File sengaja bebas tipe ObjC proyek agar bisa dikompilasi ke target Unit Tests (tanpa bridging header) — key pref diduplikasi (13–16).
- `printOperation(for:)` re-assign `operation.printPanel` karena lazy caching WebKit tak dijamin (71–77).
- `webViewWebContentProcessDidTerminate` ada khusus agar completion tak tertahan permanen saat crash WebContent (205–213).

---

## 5. `NSAlertExtension.swift` (375)

Factory `NSAlert` + normalisasi path bookmark + helper pesan alert.

### Tipe
`enum SABookmarkPathNormalizer` (11, stateless); `@objc final class SABookmarkAlertContent` (47, stateless); `@objc extension NSAlert` (75).
Literal constraint id `"SABookmarkAlertHelpWidth"`/`"Height"` (340–341, 345, 350–351).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 12–44 | | `normalizedFilePath(forBookmarkPath:)` / `displayName(forBookmarkPath:)` / `filePath(forBookmarkPath:)` | Normalisasi path |
| 48–72 | | `displayNames(forBookmarkPaths:)` / `staleBookmarksMessage(count:)` / `missingBookmarksMessage(count:)` / `displayName(forBookmarkPath:)` | Konten alert |
| 84–245 | | `createDefaultAlert` / `createDefaultAlertWithSuppression` / `createAlert` / `createWarningAlert` / `createInfoAlert` / `createAccessoryAlert` | Factory |
| 250–374 | | `createScrollableListAccessoryAlert` / `createAccessoryWarningAlert` / `scrollableListAccessoryView(listItems:helpView:)` | Alert lanjutan |

### Temuan
- Setiap factory jalan di `DispatchQueue.main.async` + `runModal()` (90, 120, 161, 186, 205, 231, 258, 286) — modal by default.
- Hanya sebagian punya `@objc(...)` eksplisit: `createWarningAlert` (182), `createScrollableListAccessoryAlert` (249); `scrollableListAccessoryView` `@nonobjc` (298); lainnya mengandalkan default `@objc extension`.
- `scrollableListAccessoryView` hitung ukuran dari `NSScreen.main` dengan clamp (299–302) + strip/re-add constraint teridentifikasi untuk hindari duplikat saat reuse (337–353).
- **UNKNOWN**: caller eksternal mana yang bergantung pada nama selector `@objc(...)` spesifik.

---

## 6. `SAKeychain.swift` (286)

Store generic-password berbasis `SecItem*` (pengganti `SPKeychain`), beroperasi pada login keychain berbasis file.

### Tipe/konstanta
`@objc final class SAKeychain: NSObject, SAKeychainProviding` (57); `private static let genericAttribute = Data("application password".utf8)` (61). Stateless.
External: `SAKeychainProviding`, `SAKeychainNaming` — **UNKNOWN** lokasi file.

### `// MARK:`
`63 (Store operations), 213 (Names and accounts), 231 (Helpers)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 65–108 | | `add(password:name:account:)` / `add(password:name:account:label:)` / `password(name:account:)` | Tambah/baca |
| 110–139 | | `deletePassword(name:account:)` / `passwordExists(name:account:)` | Hapus/cek |
| 141 | 211 | `updateItem(name:account:toName:newAccount:password:)` | Update/rename item |
| 215–229 | | `name(favoriteName:id:)` / `account(user:host:database:)` / `sshName(favoriteName:id:)` / `sshAccount(user:host:)` | Nama + account |
| 237–285 | | `persistentRef(name:account:)` / `label(forPersistentRef:)` / `isValid(name:account:)` / `presentErrorSync(title:message:)` | Helper |

### Temuan
- **Sengaja tak pernah** pakai `kSecUseDataProtectionKeychain` → menyentuh login keychain berbasis file yang sama dengan kode legacy; tanpa migrasi (`:40-45`).
- Perilaku dipin in-file: add **no-op senyap** bila item ada (72–73); update recover dari `errSecItemNotFound` via add baru (152–156) + dari `errSecDuplicateItem` via delete + retry sekali (199–202); rename meninggalkan label (184–186, 50–51).
- Update password kosong **re-create** item karena `SecItemUpdate` abaikan `kSecValueData` zero-length (diprobe di macOS 15) — membawa label (158–177).
- `delete`/`update` pin item tunggal via persistent ref (query atribut tanpa batas akan match seluruh search list keychain) (53–56, 112–115, 148–152).
- `presentErrorSync` blok thread pemanggil seperti legacy `SPMainQSync`/`runModal` (268–285).

---

## 7. `SPConstants.m` (535)

Pool global konstanta string/scalar + satu kategori `NSString` untuk parsing tipe table-view.

### Tipe/konstanta
`@implementation NSString (TableViewTypeEnumParser)` (33–55) — **satu-satunya** tipe.
`const NSUInteger` `SPNarrowDownCompletionMaxRows` (59), `SPMaxQueryLengthForWarning` (60); `const NSInteger SPBundleRedirectActionLastCode` (401); `const long SPBundleCurrentVersion` (464); array `SPViewModeToMainToolbarMap[]` (315–324); `#define OWS` (524).

### Fungsi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 35 | 53 | `-[NSString tableViewTypeEnumFromString]` | Map string tipe → `SPTableViewType` enum |

### Kategori konstanta (banner `//`)
- Narrow-down/query length (58–60); nama DB sistem (62–66); drag type (68–72); ekstensi file (74–80); content type SPF (82–86); nama file (88–97); nama folder (99–102); key filter tabel (104–105); Prefpane General/Tables/Favorites/Notifications/Network/Editor/AutoUpdate (107–193); GUI Prefs (195–217); Hidden Prefs (219–223); Import/export (225–240); token nama file export (241–251); Misc (`SPUniqueSchemaDelimiter`, notifikasi) (253–293); URL (295–299); Toolbar (300–313); array map view-mode (315–324); toolbar preferensi (326–335); MCP (337–341); key favorite (343–375); AWS IAM (377–382); Vault (384–388); misc key (390–395); import/export favorite key (397–398); Bundle Files/Editor (400–521); regex CURRENT_TIMESTAMP (524–526); URL scheme temp-path (528–532); error name (534–535).

### Temuan
- `tableViewTypeEnumFromString` bangun dict ter-cache lazy (dispatch_once), return `NSNotFound` untuk `"SPTableViewInvalid"` (37–52).
- `SASecureBookmarks` sengaja bernilai `SPSecureBookmarks` dengan warning in-source: `// MUST be SPSecureBookmarks for var name SASecureBookmarks` (236).
- `SPUniqueSchemaDelimiter = @"\uFFF8"` (278) — delimiter tak terlihat.
- `SPViewModeToMainToolbarMap` array pointer `nil`-terminated (315–324); index 0 `nil`.

---

## 8. `SPLogger.m` (261)

Singleton logger debug user-support, tulis baris bertimestamp ke file log di Desktop + opsi dump `leaks` saat terminate.

### Ivar/properti
`static SPLogger *logger = nil` (41); `@interface SPLogger ()` (43–50): `_initLogFile` (45), `_outputTimeString` (46), C `_isSPLeaksLog` (48). `@synthesize dumpLeaksOnTermination` (65), `removeOldLeakDumpsOnTermination` (66). Ivar `.h:33-54`.

### `#pragma mark`
`82/83 (Initialisation and teardown), 105/106 (Logging functions), 204/205 (Private API)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 71–103 | | `+logger` / `+allocWithZone:` / `-init` | Singleton |
| 108 | 131 | `log:...` | Format + append baris bertimestamp ke file |
| 133 | 202 | `dumpLeaks` | Jalankan `leaks`, tulis dump; opsional hapus dump lama |
| 207–247 | | `_initLogFile` | Buka/buat file log Desktop + tulis header |
| 249–254 | | `_outputTimeString` | Tulis baris waktu launch |
| 256–259 | | `_isSPLeaksLog` | Filter `scandir` match "sp.leaks" |

### Temuan
- `log:` paksa sync disk langsung (`synchronizeFile` sebelum & sesudah write, 122, 130); doc memperingatkan biaya kinerja (59–60).
- `dumpLeaks` pakai `getlogin`/`getpwnam`, fallback `/tmp` bila user info tak tersedia (142–147); tulis `~/Desktop/sp.leaks.<pid>.log` (189).
- `_initLogFile` hasilkan tiga alert "Logging error" berbeda (216–218, 223–225, 233–235).
- **UNKNOWN**: `_outputTimeString` (249–254) dideklarasikan (46) tapi **tak ada caller in-file** — dipanggil eksternal (mis. saat launch). **UNKNOWN**: interaksi singleton via superclass `SPSingleton` (`.h:33`) + `+logger`/`allocWithZone:` (71–91).

---

## 9. `SPFileHandle.m` (484)

Wrapper mirip `NSFileHandle` yang transparan membaca gzip/bzip2 dan menulis gzip/bzip2 (atau plain) via thread kompresi background.

### Ivar/konstanta
`#define SPFH_MAX_WRITE_BUFFER_SIZE 1048576` (38); `struct SPRawFileHandles { FILE*; BZFILE*; gzFile; }` (40–44); `@interface SPFileHandle ()` (46–51). Ivar `.h:41-59`. Enum `SPFileCompressionFormat` (dipakai 272, 372) — defined eksternal (**UNKNOWN**).

### `#pragma mark`
`55, 159/160 (Class methods), 212/213 (Data reading), 264/265 (Data writing), 366/367 (File information), 468`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 67 | 157 | `initWithFile:fromPath:mode:` | Setup handle, deteksi kompresi, start thread writer |
| 166–210 | | `fileHandleForReadingAtPath:` / `fileHandleForWritingAtPath:` / `fileHandleForPath:mode:` | Factory |
| 218–262 | | `readDataOfLength:` / `readDataToEndOfFile` / `realDataReadLength` | Baca (dekompresi) |
| 272 | 294 | `setCompressionFormat:` | Ganti kompresi, reopen handle |
| 300–341 | | `writeData:` / `synchronizeFile` | Tulis (buffer, blok saat penuh) |
| 347–375 | | `closeFile` / `compressionFormat` | Tutup + getter |
| 382 | 437 | `_writeBufferToData` | Thread background tulis/kompres buffer |
| 442–482 | | `_closeFileHandles` / `dealloc` | Tutup handle + free |

### Temuan
- Deteksi kompresi baca: gzip via `gzopen`/`gzdirect` (99–114), bzip2 via magic 4-byte `BZh0-9` (116–138); else plain.
- Hanya `O_RDONLY`/`O_WRONLY` didukung; mode lain → `NSInvalidArgumentException` (81–83).
- `setCompressionFormat:` raise bila dipanggil setelah data ditulis (279) setelah menutup handle sebelumnya (277).
- Write jalan di `processingThread` bernama "SPFileHandle data writing thread" (150–152); `writeData:` throttle saat `bufferDataLength > SPFH_MAX_WRITE_BUFFER_SIZE` (316–321); `synchronizeFile` spin pada `allDataWritten` (333–338).
- **UNKNOWN**: ivar `bufferPosition` (90) dan `endOfFile` (91) diinisialisasi di `initWithFile:` tapi **tak pernah dibaca/ditulis lagi** di file ini — tampak dead/unused.

## Ringkasan lintas-file
- **Semantik sync main-thread** berulang: `SPMainQSync` (`SPFunctions.m:59`), `SAKeychain.presentErrorSync` (`SAKeychain.swift:271`), dan setiap factory `NSAlertExtension` `runModal` di main.
- Factory `@objc` di `NSAlertExtension.swift` dikonsumsi ObjC (`SPLogger.m:218/225/235` panggil `createWarningAlertWithTitle:...`).
- Duplikasi konstanta: `printBackgroundPreferenceKey` (`SAPrintUtility.swift:16`) harus sinkron `SPPrintBackground` (`SPConstants.m:267`).
- Jalur tak tersentuh: `SPLogger._outputTimeString` + `SPFileHandle.bufferPosition`/`endOfFile` tanpa pemakaian in-file.
