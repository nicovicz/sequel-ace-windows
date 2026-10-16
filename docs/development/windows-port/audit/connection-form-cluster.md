# Audit: Connection-form / favorites Swift cluster

Audit method-by-method klaster `ConnectionView/`. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `SAConnectionFormModel.swift` (540), `SAConnectionDetailsValidator.swift` (320),
`SAFavoriteDuplicateMatcher.swift` (214), `SAFavoritesListDataSource.swift` (449).

---

## 1. `SAConnectionFormModel.swift` (540)

Model `ObservableObject` di belakang form koneksi SwiftUI; membungkus value-type `SAConnectionInfo`, menjembatani Vault mount/role, menurunkan nilai tampilan, dan memusatkan validasi pra-koneksi.

### Tipe & konstanta
`enum SATimeZoneChoice` (31–67, Hashable; case `.server`/`.system`/`.fixed(String)`); `enum SATimeZoneMenuEntry` (72–96); `final class SAConnectionFormModel: ObservableObject` (98–540); flag privat `isReplacingInfo` (117).

### `// MARK:`
`24 (Time Zone Choice), 127 (Vault mount / role), 210 (Loading a favorite), 257 (Derived display values), 307 (Time zone), 321 (Flag bridges), 375 (Per-type form shape), 407 (Type changes), 438 (Identity changes), 469 (Validation)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 39–66 | | `SATimeZoneChoice.init(mode:identifier:)` / `mode` / `identifier` | Rekonstruksi choice; id fixed kosong → `.server` |
| 77 | 95 | `SATimeZoneMenuEntry.menu(identifiers:)` | Bangun menu urut, separator saat prefix region berubah |
| 105–125 | | `info` (@Published) / `isReplacingInfo` / `replaceInfo(_:)` | Field + replace wholesale tanpa dihitung edit |
| 136–159 | | `vaultMount` / `vaultCredentialsRole` / `blankFormInfo` / `init(info:)` | Vault halves + init |
| 169–208 | | `commitVaultCredentialsRole()` / `rejoinVaultCredentialsPath()` / `resplitVaultCredentialsPathIfChangedExternally()` / `convenience init(objc:)` / `apply(to:)` | Vault split + bridge ObjC |
| 221–255 | | `load(favorite:)` / `string(_:)` / `loadQuickConnect()` | Load favorite + quick connect |
| 263–305 | | `effectiveName` / `generatedName` / `canAttemptConnection` | Nilai turunan |
| 313–373 | | `timeZoneChoice` / `flag(_:)` / `setFlag(_:_:)` / `useSSL` / `allowDataLocalInfile` / `enableClearTextPlugin` / `requestServerPublicKey` / `sshKeyLocationEnabled` / 3 SSL enabled bridge | Bridge flag Int↔Bool |
| 381–405 | | `forcesSSL` / `showsSSLToggle` / `showsSSLFileOptions` / `showsClearTextPluginToggle` / `showsRequestServerPublicKeyToggle` | Bentuk form per-tipe |
| 423–467 | | `syncTypeDerivedState(from:)` / `clearPasswordsIfIdentityWasEdited(from:)` | Reaksi perubahan tipe/identitas |
| 474–539 | | `validate()` / `validateVaultDetails()` | Validasi |

### Temuan
- **Urutan didSet `info`** (106–110): `resplitVaultCredentialsPathIfChangedExternally`, `clearPasswordsIfIdentityWasEdited`, `syncTypeDerivedState` — urutan tetap.
- **Rasional split Vault** (129–134, 161–168): `credPath(mount:role:)` return "" saat role kosong → halves **disimpan**, bukan computed.
- **Blank Quick Connect** (144–147, 246–252): blank = color −1, compression on, AWS profile "default"; `SAConnectionInfo()` mentah tidak cocok.
- **Carve-out Vault favorite** (224–231): `load(favorite:)` manual membawa `vaultPort`/`vaultOIDCMount` karena decoder D1 mengabaikannya.
- **`canAttemptConnection` Vault triple** (298–303): butuh host DAN `vaultHost` DAN `vaultCredentialsPath` non-blank.
- **IAM SSL tak di-restore** (430–435): clear `useSSL` untuk IAM sengaja tak dibatalkan saat switch balik (mirror AppKit).
- String alert `validateVaultDetails` (504–536): title `"Insufficient connection details"`; urutan cek vaultHost → credsPath → host DB (511, 520, 529).
- **UNKNOWN**: set field/default `SAConnectionInfo` + defaulting "D1" `SAConnectionInfoObjC.info(fromFavoriteDictionary:)` (di luar file; hanya dirujuk 149, 222); case `SAConnectionTimeZoneMode` (41–46).

---

## 2. `SAConnectionDetailsValidator.swift` (320)

Validator pra-koneksi murni Foundation (bebas AppKit), diangkat dari `-[SPConnectionController initiateConnection:]` (Fase D3) + aturan kind file/PEM chooser.

### Tipe & konstanta
`@objc enum SAConnectionValidationFailureKind: Int` (22–34, 8 case); `@objc final class SAConnectionValidationFailure` (39–50); `SAConnectionDetailsValidator` (52–186); `@objc enum SAConnectionFileKind: Int` (196–225, 4 case); `struct SAConnectionFileRejection` (229–232); `SAConnectionFileValidator` (237–320).
Penanda PEM `"PRIVATE KEY-----"` (208), `"CERTIFICATE-----"` (209); set ekstensi ditolak `["pub"]` untuk SSH key (221).

### `// MARK:`
`188 (Chooser file kinds)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 40–49 | | `kind` / `alertTitle` / `alertMessage` / init | Bundel kind + string alert |
| 68 | 178 | `validate(type:host:sshHost:...)` | Cek pra-koneksi berurut; return kegagalan pertama |
| 183 | 185 | `fileExistsExpandingTilde(_:)` | Tilde-expand + cek `fileExists` |
| 206–224 | | `requiredPEMMarker` / `rejectedExtensions` | Aturan kind file |
| 230–231 | | `alertTitle` / `alertMessage` | Rejection |
| 245–289 | | `isValidPEM(contents:kind:)` / `rejection(contents:kind:fileName:)` | Cek PEM |
| 294 | 319 | `rejection(forFileAt:kind:)` | Baca file; tolak ekstensi/unreadable/PEM buruk |

### Aturan/string validasi
- **Rule 1 — host wajib** (83–98): `.tcpIP`, `.sshTunnel` (kecuali remote-socket path diset), `.awsIAM`. Title `"Insufficient connection details"`, pesan `"Insufficient details provided to establish a connection. Please enter at least the hostname."`.
- **Rule 2 — SSH host** (100–110): `.sshTunnel` + ssh host blank → `.sshHostMissing`.
- **Rule 3 — SSH key ada** (112–123): `.sshTunnel` + `sshKeyLocationEnabled` + path + `!fileExistsExpandingTilde` → `.sshKeyFileMissing`; title `"SSH Key not found"`.
- **Rule 4–6 — file SSL** (139–175): gated `type ∈ {tcpIP, socket, sshTunnel, vault}` DAN `useSSL`. Cek key → cert → CA cert (`.sslKeyFileMissing`/`.sslCertificateFileMissing`/`.sslCACertFileMissing`).
- **Fix terdokumentasi** (129–138): `.sshTunnel` dulu dihilangkan dari gate SSL-file — kini diperbaiki.
- **PEM check** (239–257): split `.newlines`; butuh baris dengan marker + `-----BEGIN` dan marker + `-----END`. Kind tanpa marker → `true` (246).
- **Rejection titles** (269–288): `"“%@” is not a valid private key file."` (sslKey), `"“%@” is not a valid client certificate file."` (sslCertificate). `.sshKey`/`.sslCACert` ditandai unreachable (285–287).
- **Tolak `.pub`** (294–303): dicek sebelum baca file; title `"“%@” is a public key."`.
- **File unreadable** (307–313): `error.localizedDescription` + `localizedRecoverySuggestion`; decode ASCII `?? ""` (315–317).
- **UNKNOWN**: apakah `SAConnectionValidationFailureKind` di-switch exhaustif di tempat lain (tak ada switch di file ini).

---

## 3. `SAFavoriteDuplicateMatcher.swift` (214)

Logika deteksi duplikat murni Foundation untuk import koneksi, diangkat dari `SPConnectionController` + map type string↔tag, normalisasi port, merge update.

### Tipe & konstanta
`@objc final class SAFavoriteDuplicateMatcher` (19–214); `enum FavoriteKey` privat (23–38).
Tag `tcpIPTag = 0` (43), `socketTag = 1` (44), `sshTunnelTag = 2` (45), `awsIAMTag = 3` (46) — mirror `SPConnectionType` di `SPConstants.h` (42).

### `// MARK:`
`21 (Favorite keys (keep in sync with SPConstants.m)), 40 (Connection-type mapping), 84 (Port normalization), 110 (The duplicate predicate), 162 (Applying an update to a duplicate), 179 (Lenient value readers)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 55–82 | | `typeTag(forString:)` / `typeString(forTag:)` | Map tipe↔tag, default TCP/IP |
| 93 | 108 | `normalizedPort(_:typeString:)` | Normalisasi port NSNumber/String; kosong → "3306" |
| 126 | 160 | `favorite(_:isDuplicateOfHost:user:database:port:typeString:modeSpecificFields:)` | Predikat duplikat inti |
| 170 | 177 | `mergedFavorite(from:applying:)` | Terapkan newData ke existing, pertahankan name+id |
| 185–213 | | `string(_:)` / `intValue(_:)` / `candidateValue(_:urlKey:favoriteKey:)` | Reader longgar |

### Aturan matcher
- **Base equality** (135–140): host, user, database, port ternormalisasi, tipe koneksi semua match; port banding pakai type string masing-masing sisi (138–139).
- **Quirk Vault** (48–53, 68–69): Vault tak punya string → map ke TCP/IP dua arah → **dibanding sebagai TCP/IP** untuk duplikat — sengaja dipertahankan.
- **Field per-tipe bila `modeFields` ada** (144–159): SSH tunnel `sshHost`/`sshUser`/`sshPort`/`sshRemoteSocketPath`; socket `socket`; AWS IAM `awsRegion`/`awsProfile`; default (termasuk TCP/IP + Vault) `true` (157–158).
- **`modeSpecificFields == nil`** (144): skip komparasi per-tipe (dipakai jalur URL-import) (122–124).
- **Dual-key read** (120–122, 209–213): baca bawah URL key dulu, lalu plist key.
- **`mergedFavorite`** (162–176): name dan id di-skip (173) agar "Update" pertahankan identitas/nama keychain + nama user.
- **Rasional `intValue`** (189–197): pakai `NSString.integerValue` bukan `Int(_:)` karena berbeda pada input non-kanonik (" 2"→2, "2x"→2); tipe SSH yang dipersist tak boleh jadi TCP/IP (cited "Codex, #2583").
- **Port normalisasi** (84–107): port socket dibiarkan; lain default kosong → "3306".
- **UNKNOWN**: call site mana yang kirim `modeSpecificFields` vs nil (dua jalur import dirujuk 122).

---

## 4. `SAFavoritesListDataSource.swift` (449)

Datasource + delegate `NSOutlineView` untuk sidebar favorites, diekstrak dari `SPConnectionController`; memiliki tree favorites, baris Quick Connect, filter, drag & drop, tampilan sel, delegasi intent ke `SAFavoritesListDelegate`.

### Tipe & konstanta
`SAFavoritesListDataSource` (24–160) + extension `NSOutlineViewDataSource` (164–304) + `NSOutlineViewDelegate` (308–449).
Konstanta file-private (14): `kSPDatabaseImage = "database-small"` (15), `kSPQuickConnectImage = "quick-connect-icon.pdf"` (16), `kSPQuickConnectImageWhite = "quick-connect-icon-white.pdf"` (17).

### `// MARK:`
`26 (Properties), 59 (Initialization), 83 (Public API), 116 (Filtering), 162 (NSOutlineViewDataSource), 211 (Drag & Drop), 306 (NSOutlineViewDelegate), 441 (Private Helpers)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 29–57 | | `favoritesRoot` / `quickConnectItem` / `quickConnectCell` / `folderImage` / `draggedNodes` / `delegate` / `favoritesController` / `searchQuery` / `visibleNodes` | Properti |
| 61 | 81 | `init(favoritesRoot:favoritesController:)` | Bangun node/cell Quick Connect + folder icon |
| 86–114 | | `attach(to:)` / `reloadData(in:)` | Pasang datasource/delegate + reload |
| 122–159 | | `rebuildVisibleNodes()` / `filteredChildren(of:)` / `restoreOutlineViewState(_:in:)` | Filter + restore expand |
| 166–209 | | `numberOfChildrenOfItem:` / `child:ofItem:` / `objectValueFor:byItem:` / `setObjectValue:for:byItem:` | Datasource (rename di 203) |
| 213–303 | | `writeItems:to:` / `validateDrop:...` / `acceptDrop:item:childIndex:` | Drag & drop |
| 310–439 | | `isGroupItem:` / `outlineViewSelectionIsChanging:` / `SelectionDidChange` / `dataCellFor` / `willDisplayCell` / `heightOfRowByItem` / `toolTipFor` / `shouldSelectItem` / `isItemExpandable` / `shouldShowOutlineCellForItem` / `shouldCollapseItem` / `shouldEdit:item:` / `ItemDidCollapse` / `ItemDidExpand` | Delegate |
| 443 | 448 | `setNodeIsExpanded(_:from:)` | Set `SPGroupNode.nodeIsExpanded` dari notification |

### Temuan
- **Root "+1 Quick Connect"** (170–172, 180–186): count root +1; index 0 = `quickConnectItem`, lainnya digeser +1.
- **Filtering** (118–129): aturan match di `SAFavoriteSearchMatcher`/`SAFavoriteSearchTreeWalker`; `visibleNodes = nil` mematikan filter.
- **Re-expand setelah reload** (93–113): grup top-level tanpa disclosure triangle di source-list mode → snapshot item **sebelum** expand agar index tak bergeser.
- **Deteksi top-level**: `node.parent?.parent == nil` (150, 312, 338, 370, 398, 411, 416).
- **Restriksi drag** (213–230): abort bila `favoritesListShouldBeginDrag?()` false; tolak drag item root-level (222); tulis pasteboard type `SPFavoritesPasteboardDragType`.
- **Validasi drop** (232–252): tolak target `quickConnectItem` (234–235), drop-on non-group (238), circular via walk ancestor vs `draggedNodes` (240–245); hanya internal drag `.move`.
- **acceptDrop reorder** (254–303): drop-on-group sisip index 0 + expand; index disesuaikan saat drag turun (278); relokasi dari parent lama saat cross-parent (280); `favoritesController.saveFavorites()` + `delegate?.favoritesListDidReorderNodes?()`; restore seleksi via `NSMutableIndexSet` (294–300).
- **Rename** (203–208): trim, tolak kosong, forward ke `favoritesListDidRenameNode(_:to:)`.
- **Cell display** (334–355): Quick Connect top-level pakai ikon putih/warna by seleksi; grup `folderImage`; favorite `kSPDatabaseImage` + warna dari `SPFavoriteColorSupport.sharedInstance().color(for:)` key `SPFavoriteColorIndexKey`.
- **Magic string tinggi baris** (359): `"{ǞṶḹÜ∑zgyf"` sebagai probe metrik; min 24.0 (360).
- **Format tooltip** (378–390): `"%lu favorite"`/`"%lu favorites"` + `"%lu group"`/`"%lu groups"`; grup `"<name> - <parts join ", ">"`; favorite `"<name> (<host>)"` atau nama saja bila host kosong.
- **Guard edit** (422–427): blok edit saat keyDown `NSBackTabCharacter` + `(outlineView as? SPFavoritesOutlineView)?.justGainedFocus == true`.
- **UNKNOWN**: siapa memanggil `restoreOutlineViewState`/`attach`/`reloadData`; semantik `SPFavoritesOutlineView.justGainedFocus`; protokol `SAFavoritesListDelegate` penuh; `SPFavoriteSearchMatcher`/`SAFavoriteSearchTreeWalker`.

## Ringkasan lintas-file
- **Validasi Vault dipisah** by design: `SAConnectionDetailsValidator` mengabaikan precondition Vault (dibagi dengan controller AppKit yang jalankan sendiri), jadi `SAConnectionFormModel.validateVaultDetails()` memunculkan `vaultHostMissing`/`vaultCredentialsPathMissing` sendiri (`SAConnectionDetailsValidator.swift:29–33`).
- **Form model satu-satunya pemanggil** validator D3 (pass flag Int sebagai `!= 0`) (`SAConnectionFormModel.swift:479–493`).
- **Filosofi defaulting bersama**: form model + matcher sama-sama pakai reader NSNumber/String longgar.
- **Equivalensi Vault↔TCP-IP** berulang: matcher memperlakukan Vault sebagai TCP/IP (48–53), sementara di tempat lain `.vault` adalah case kelas satu (validator gate SSL 139; `canAttemptConnection` 298).
