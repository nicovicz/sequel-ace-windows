# Audit: Model cluster

Audit method-by-method klaster `Source/Model/`. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `SAConnectionInfo.swift` (410), `SAConnectionInfo+Favorite.swift` (264),
`SAConnectionInfo+ConnectionString.swift` (250), `ConnectionStringParser.swift` (206),
`SATooltipDismissal.swift` (328), `SPTooltip.m` (529), `TreeNodes/SPTreeNode.m` (336).

---

## 1. `SAConnectionInfo.swift` (410)

Value-type Swift `struct SAConnectionInfo` (semua parameter koneksi MySQL) + wrapper `@objc` `SAConnectionInfoObjC`.

### Tipe & konstanta
`@objc enum SAConnectionType: Int` (14–20: tcpIP 0, socket 1, sshTunnel 2, awsIAM 3, vault 4);
`@objc enum SAConnectionTimeZoneMode: Int` (25–29: useServerTZ/useSystemTZ/useFixedTZ);
`struct SAConnectionInfo` (35–115); `@objc class SAConnectionInfoObjC: NSObject` (121–410).
Konstanta `@objc static let keychainPasswordPlaceholder = "SequelAceSecretPassword"` (123).

### `// MARK:`
`11, 22, 31, 37 (Basic Connection), 50 (Time Zone), 55 (Special Settings), 61 (AWS IAM), 67 (Vault), 74 (SSL), 84 (SSH Tunnel), 108 (Keychain), 117 (ObjC Bridging), 190, 242, 254, 271, 288, 310, 347, 384`.

### Definisi
Stored property struct (39–114): `type`, `name`, `host`, `user`, `password`, `database`, `socket`, `port`, `colorIndex`, `useCompression`, `timeZoneMode`, `timeZoneIdentifier`, `allowDataLocalInfile`, `enableClearTextPlugin`, `requestServerPublicKey`, `useAWSIAMAuth`, `awsRegion`, `awsProfile`, `vaultHost`, `vaultPort`, `vaultOIDCMount`, `vaultCredentialsPath`, `useSSL` + 3 pasang SSL enabled/path, blok SSH (86–91), `sshRemoteSocketPath` (106), blok keychain (110–114).

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 95 | 104 | `sshPortOverride: Int?` | Computed; blank→0, invalid→nil, valid→port |
| 127–135 | | `init()` / `init(info:)` | Init wrapper |
| 138 | 164 | `resolvedMySQLConnectHost(for:)` | Host klien per tipe koneksi |
| 170 | 188 | `shouldDeferMySQLPasswordToDelegate(...)` | Apakah delegate memasok password per connect |
| 192–409 | | ~35 bridge `var` get/set ke `info.*` | Forwarding murni (mechanical) |

### Temuan
- `resolvedMySQLConnectHost` nil untuk socket, pertahankan loopback eksplisit untuk SSH tunnel, else fallback host (`144–159`); default `"127.0.0.1"` untuk host kosong (141).
- `shouldDeferMySQLPasswordToDelegate` **selalu** defer AWS IAM (token 15 menit, doc 166–168); **tidak pernah** untuk Vault; lainnya hanya bila nama item keychain diset **dan** password == placeholder (180–186). **UNKNOWN**: caller pakai `==` mentah terhadap konstanta placeholder.
- Hanya `sshPortOverride` computed (95); tak ada `Equatable`/`Codable`/`Hashable`. **UNKNOWN** bagaimana caller membandingkan dua `SAConnectionInfo`.
- `@unknown default` pada kedua switch (161–162).

---

## 2. `SAConnectionInfo+Favorite.swift` (264)

Decode dict plist favorite → `SAConnectionInfo` (Phase D1) + template/duplikasi dict favorite (Phase D2) via bridge ObjC.

### Tipe & konstanta
`private enum FavoriteKey` (35–72) — key string inline, **harus sinkron** dengan `SPConstants.m` (15–20).
Extension `SAConnectionInfo` (100–168) + `SAConnectionInfoObjC` (172–263).

### `// MARK:`
`33 (Favorite dictionary keys), 74 (Lenient value readers), 98 (SAConnectionInfo decoding), 170 (ObjC bridge), 197 (Favorite dictionary templates Phase D2)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 80–96 | | `intValue(_:default:)` / `boolValue(_:default:)` / `stringValue(_:default:)` | Koersi longgar |
| 107 | 167 | `fromFavoriteDictionary(_:)` | Decode dict favorite → info dengan default historis |
| 178–194 | | `info(fromFavoriteDictionary:)` / `rawFavoriteString(_:)` | Entry ObjC + koersi |
| 209 | 245 | `defaultNewFavoriteDictionary(withID:)` | Dict plist favorite baru |
| 252 | 263 | `duplicatedFavoriteDictionary(fromFavorite:withID:)` | Duplikat dict, id baru + nama "Copy" |

### Temuan
- Aturan default historis di-encode di sini: `colorIndex` −1 (121), `useCompression` **true** (124), `awsProfile` `"default"` (143). Doc mencatat ini dulu inline di `updateFavoriteSelection:` (8–13).
- `useAWSIAMAuth` **diturunkan**: `type == .awsIAM ? 1 : 0` (141).
- Time-zone identifier hanya dipertahankan di mode fixed (131–133).
- **Sengaja tidak didekode**: password/sshPassword/nama keychain + `vaultPort`/`vaultOIDCMount` (komentar 22–28, 55, 145). **UNKNOWN**: form pakai nil (absen) → non-optional string `SAConnectionInfo` → decoded bawaan `""`.
- Kuirk template dipertahankan (203–207): tanpa key `useCompression`, tanpa key *path* SSL (hanya flag enabled), vaultPort/OIDCMount `""`.
- `defaultNewFavoriteDictionary` tulis `type` = 0, `colorIndex` = −1 (213, 217); nama localized `"New Favorite"` (212).

---

## 3. `SAConnectionInfo+ConnectionString.swift` (250)

Generate `mysql://` dari `SAConnectionInfo`, wrapper ObjC, dan `SPFavoriteNode` (dict favorite).

### Tipe & konstanta
`extension SAConnectionInfo` (10–112); `SAConnectionInfoObjC` (114–124); `SPFavoriteNode` (126–250).
Skema `"mysql"` (20, 138); host fallback `"127.0.0.1"` (31, 167).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 18 | 111 | `toConnectionString(includePassword:includeSSHKeyPath:)` | Bangun URL mysql:// dari value-type |
| 121 | 123 | `SAConnectionInfoObjC.toConnectionString(...)` | Wrapper ObjC |
| 132 | 249 | `SPFavoriteNode.toConnectionString(includePassword:)` | Bangun URL dari dict favorite |

### Temuan
- Format `mysql://[user[:password]@]host[:port][/database][?params]` (12–13).
- Mapping tipe→query (48–96): socket→`type=socket`+`socket`; sshTunnel→`type=ssh`+ssh_* (password hanya bila `includePassword`); awsIAM→`type=aws_iam`+aws_*. `tcpIP` dan `vault` **tidak emit type** (87–92) — kuirk vault "selalu round-trip sebagai tcpip", rujuk `SAFavoriteDuplicateMatcher.typeTag(forString:)` (90–91).
- Flag `enable_cleartext_plugin=1` (100), `get_server_public_key=1` (103).
- Varian favorite-node baca password/SSH dari keychain via `SAKeychainAccess` saat `includePassword` (146–161, 206–215); host dinormalisasi `"localhost"` untuk lookup keychain saat `typeTag == 1` (socket) (154).
- Tag tipe ajaib `1`/`2`/`3` (socket/SSH/AWS) (185, 191, 224) — bukan enum `SAConnectionType`; risiko konsistensi. **UNKNOWN**: tak ada konstanta bersama.
- AWS region/profile dibaca dengan key literal `"awsRegion"`/`"awsProfile"` (226, 229) — bukan konstanta SPFavorite — asimetri.
- `components.port = Int(port)` bisa nil bila port non-numerik (36); tanpa validasi.

---

## 4. `ConnectionStringParser.swift` (206)

Parser terpusat `mysql://` → `ConnectionStringParseResult` (dict detail + autoConnect + invalid params).

### Tipe & konstanta
`@objc class ConnectionStringParseResult` (12–24); `@objc class ConnectionStringParser` (27–206).
`validQueryParameters` (30–35); host default `"127.0.0.1"` (55); string tipe internal `"SPAWSIAMConnection"` dst (165–171).

### `// MARK:`
**Tidak ada.**

### Definisi
`ConnectionStringParseResult` fields (13–16) + init (18–23);
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 30 | 35 | `validQueryParameters` | Whitelist nama parameter query |
| 40 | 177 | `parse(_:)` | Parse URL → result |
| 181 | 194 | `connectionTypeString(from:)` | Map `type` URL → nama kelas internal |
| 199 | 205 | `validateConnectionString(_:)` | URL iff skema mysql |

### Temuan
- Scheme gate: gagal kecuali skema lowercase `"mysql"` (41–43); `validateConnectionString` juga tolak non-mysql (200–202).
- Key detail terisi: `host`, `user`, `password`, `port`, `database`, `type`, `socket`, `ssh_host`, `ssh_port`, `ssh_user`, `ssh_password`, `ssh_keyLocationEnabled`, `ssh_keyLocation`, `ssh_remote_socket_path` **dan** `sshRemoteSocketPath` (132–133), `aws_region`, `aws_profile`, plus `enableClearTextPlugin` + `requestServerPublicKey` (144, 148).
- Adanya password memaksa `autoConnect = true` (63–68).
- Boolean flag terima `true`/`1`/`yes`/`y` (143, 147).
- `type` tak dikenal → `invalidParameters` (106–108); key tak dikenal juga (93–95).
- Inferensi tipe bila `type` absen (157–173): AWS → `SPAWSIAMConnection`, socket → `SPSocketConnection`, SSH → `SPSSHTunnelConnection`, else `SPTCPIPConnection`.
- `connectionTypeString` kenali `socket`, `ssh`, `aws_iam`/`awsiam`, `tcpip`/`tcp` (183–190); **UNKNOWN**: tak ada mapping `vault`.

---

## 5. `SATooltipDismissal.swift` (328)

Logika testable Swift yang diekstrak dari SPTooltip: dismissal policy murni, monitor event lokal, siklus tooltip, dan state pengukuran per-web-view.

### Tipe & konstanta
`SATooltipDismissalPolicy` (13–47) dengan `ignorePeriod = 0.05` (16), `moveThreshold = 10` (19);
`SATooltipDismissalMonitor` (55–140); `SATooltipLifecycle` (146–255); `SATooltipMeasurement` (263–327) default height 21 (269) / width 400 (271).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 22–46 | | `closesImmediately(_:)` / `evaluateMouseMove(...)` | Keputusan dismissal (murni) |
| 67–139 | | `init(keyWindow:onClose:)` / `handle(_:)` / `close()` / `stop()` / `deinit` | Monitor event lokal |
| 151–254 | | `isMonitoring` / `prepareForNewTooltip` / `beginDismissalMonitoring` / `detachDismissalMonitor` / `fadeMayContinue` / `tooltipDidClose` / `isCurrent` / `shouldCloseAfterNavigationFailure` | Siklus tooltip |
| 269–327 | | `height` / `width` / `init(webView:)` / `recordHeight` / `recordWidth` / `isComplete` / `applies(toCurrentWebView:)` / `shouldKeepWaiting` | Pengukuran |

### Temuan
- `closesImmediately` trigger keyDown, semua mouse down, scrollWheel (24); gerakan mouse ditangani terpisah dengan grace+threshold (36–45).
- Monitor menggantikan loop `nextEventMatchingMask:` lama (doc 49–54); pakai `addLocalMonitorForEvents` + kembalikan event agar dispatch normal (90–93).
- Key-window: bila ada key window → `acceptsMouseMovedEvents = true` + restore di `stop()` (73–74, 128–130); observer `didResignKey` (75), bila tak ada key window saat show → observer `didBecomeKey` menutup (82); `didResignActive` juga menutup (86).
- `close()` guard double-fire dengan nil `onClose` di `stop()` (113, 121).
- `prepareForNewTooltip` reset `tooltipCount = 1` (rasional 161–169); return `isVisible || isFading` (179).
- **UNKNOWN**: `openedAt` di-capture saat monitor init (61) sehingga grace period jalan dari awal monitoring, bukan frame pertama terlihat.

---

## 6. `SPTooltip.m` (529)

Singleton `NSWindow` ObjC yang merender tooltip text/html/image di WKWebView dan mendelegasikan keputusan dismissal/lifecycle ke kelas Swift.

### Ivar/konstanta
Header ivar `.h:34–41`: `wkWebView`, `animationTimer`, `animationStart`, `SPTooltipPreferencesIdentifier`. Class extension `.m:72–85` + property `lifecycle` (83). Singleton `.h:43`. Easing `slow_in_out` (64–70); warna default `#F0F0F0` (370); font default Lucida Grande size 10 (196, 201); `NSStatusWindowLevel` (266).

### `#pragma mark`
**Tidak ada** — banner `// = Setup/teardown =` (112), `// = Webview =` (347), `// = Event handling =` (460), `// = Animation =` (478).

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 64 | 70 | `slow_in_out` | Easing sigmoid untuk fade |
| 89–144 | | `+sharedInstance` / `-init` / 5 `+showWithObject:` varian | Entry show |
| 146 | 257 | `showWithObject:atLocation:ofType:displayOptions:` | Inti show: bangun konten text/html/image |
| 259–314 | | `initMeWithOptions:` / delegate navigasi / `dealloc` / `+setDisplayOptions:` | Setup + webview |
| 321 | 345 | `+caretPosition` | Titik layar caret atau mouse |
| 350–457 | | `setContent:withOptions:` / `sizeToContent` / `runInBackground:` / `wakeUpMainThreadRunloop:` | Konten + pengukuran |
| 469–526 | | `beginDismissalMonitoring` / `orderOut:` / `animationTick:` / `stopAnimation:` | Dismissal + fade |

### Temuan
- `showWithObject:...` defer keputusan ganti ke `prepareForNewTooltipWhileVisible:fading:` (151–154).
- Text path (159–212): konten non-NSString → dict error (171–172); teks kosong di-log (185–187); deteksi unix timestamp (178–182); HTML-escape `&`/`<` + wrap `<pre>` + font (198–203); fallback `"Error"` (209).
- Image path (216–250): downsize max 200pt sisi terpanjang (228–238), `NSImageView` content view, order front + begin monitoring.
- **`sizeToContent`** (379): bangun `SATooltipMeasurement` per call, set lebar temp besar `screenWidth - screenWidth/3` (395), dua `evaluateJavaScript` async (397, 404), lalu busy-wait di run loop didorong nudge thread background (414–422). Lebar tambah `+1` fudge (408–410).
- Reentrancy: guard `isCurrentWebView:` di awal + setelah wait (284, 289, 299, 304, 424).
- `orderOut:` detach monitor (484), nil `contentView`+`wkWebView` (488–490) agar callback navigasi telat tak re-show, lalu start timer fade via KVC `setValue:forKey:` (496–497).
- **UNKNOWN**: `SPTooltipPreferencesIdentifier` (`.h:40`) dideklarasikan tapi tak dirujuk di `.m`. `+setDisplayOptions:` (316) effectively dead (body dikomentari). `animationStart`/`animationTimer` diakses via KVC string (496–497, 522–523).

---

## 7. `SPTreeNode.m` (336)

Subclass `NSTreeNode` + traversal favorites, representasi dict plist untuk save, dan `NSCoding`.

### Ivar/konstanta
Header `.h:40` `BOOL isGroup`; property `childNodes` readonly nullable + `isGroup` readwrite (`.h:43,48`); `<NSCoding>` (`.h:38`). `.m`: `@dynamic childNodes` (40), `@synthesize isGroup` (41).
Konstanta `static NSString *SPTreeNodeIsGroupKey = @"SPTreeNodeIsGroup"` (36); key serialisasi `SPFavoritesGroupNameKey`/`SPFavoritesGroupIsExpandedKey`/`SPFavoriteChildrenKey` (285–287).

### `#pragma mark`
`43/44 (Initialisation), 60/61 (Public API), 293/294 (Coding protocol methods), 312/313 (Other)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 46–58 | | `+treeNodeWithRepresentedObject:` / `-initWithRepresentedObject:` | Factory + init |
| 69 | 82 | `removeObjectFromChildren:` | Hapus rekursif object identik dari child |
| 89 | 103 | `descendants` | Array depth-first semua descendant |
| 110–145 | | `childLeafs` / `allChildLeafs` | Leaf langsung / rekursif |
| 152 | 164 | `groupChildren` | Child yang berupa group |
| 173 | 197 | `parentFromArray:` | Cari parent receiver dalam array node |
| 208–251 | | `isDescendantOfOrOneOfNodes:` / `isDescendantOfNodes:` | Predikat keturunan |
| 258 | 291 | `dictionaryRepresentation` | Plist dict favorite atau subtree group |
| 296–310 | | `initWithCoder:` / `encodeWithCoder:` | Kode `isGroup` |
| 315 | 334 | `description` | Dump teks rekursif |

### Temuan
- Serialisasi **dual-mode** (258–290): node `representedObject` `SPFavoriteNode` → **salinan** `[object nodeFavorite]` (264–266); `SPGroupNode` → dict dengan `SPFavoritesGroupNameKey` (root localized `"Favorites"` bila `parentNode` nil, else `[object nodeName]`), `SPFavoritesGroupIsExpandedKey`, `SPFavoriteChildrenKey` rekursif (270–287); representasi child nil di-skip (276–278).
- **UNKNOWN**: dict untuk jenis objek lain tetap `nil` (260, 290).
- Predikat struktural pakai perbandingan identitas (`node == object` 73; `node == self` 179, 212, 240) + `indexOfObjectIdenticalTo:` (181) — pointer equality, bukan `isEqual:`.
- `parentFromArray:` rekursi hanya ke group child (186–193), stop saat capai self (179).
- `NSCoding`: hanya bool `isGroup` di-archive (302, 309); **UNKNOWN** apakah represented object/children di-archive di tempat lain.

## Ringkasan lintas-file
- **Semantik tipe terduplikasi**: `SAConnectionType` (`SAConnectionInfo.swift:14`) vs string nama kelas (`ConnectionStringParser.swift:184-190`) vs tag integer (`SAConnectionInfo+ConnectionString.swift:185-224`) — tiga encoding paralel untuk konsep yang sama.
- **Konsistensi round-trip URL**: parser mengisi `ssh_remote_socket_path` **dan** `sshRemoteSocketPath` (131–133), sedangkan generator hanya emit snake_case (`:75, :221`); versi camelCase tak punya produsen. **UNKNOWN** konsumennya.
- **Tooltip refactor**: logika dismissal diekstrak penuh & murni di Swift, SPTooltip.m menyimpan operasi window/WebView.
