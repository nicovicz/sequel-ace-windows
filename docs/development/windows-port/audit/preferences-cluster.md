# Audit: Preferences cluster

Audit method-by-method klaster preferensi. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `SPPreferenceController.m` (352, disingkat **PC**), `Panes/SPNetworkPreferencePane.m` (776, **NET**),
`Panes/SPFilePreferencePane.m` (601, **FILE**), `Panes/SPGeneralPreferencePane.m` (258, **GEN**),
`Panes/SPMCPPreferencePane.swift` (377, **MCP**). Base `SPPreferencePane` menyediakan ivar `prefs`
(`SPPreferencePane.h:43`) + `modifyAndReturnBookmarkHelpView` (`.h:46`).

---

## 1. `SPPreferenceController.m` (PC)

Controller window preferensi — toolbar, registry pane berurut, switching pane, dispatch font lintas-pane.

### Ivar
Tak ada ivar di `.m`; di header + extension. `@interface SPPreferenceController () <NSWindowDelegate>` PC:41, `_resizeWindowForContentView:` PC:43. `@synthesize` PC:51–58 (7 pane + `fontChangeTarget`). Header ivar `.h:51–75`: 7 outlet pane (51–57, `mcpPreferencePane` bukan IBOutlet), `appearancePopUp` 58, `appearanceMacOSVersionLabel` 59, `toolbar` 61, `preferencePanes` 62, toolbar item 65–70, `fontChangeTarget` 71, `@package networkItem/fileItem` 74–75. Property readonly `.h:79–85`, readwrite `fontChangeTarget` `.h:90`. **Tak ada konstanta file-local.**

### `#pragma mark`
`47, 97/98 (Toolbar item IBAction methods), 127/128 (Other), 160/161 (Private API), 260 (SPPreferenceControllerDelegate), 262 (Window delegate methods), 277/278 (Toolbar delegate methods)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 60 | 69 | `init` | Muat nib Preferences; instansiasi pane MCP programatik; zero font target |
| 74 | 95 | `windowDidLoad` | Bangun toolbar, gate popup appearance by OS, susun `preferencePanes` |
| 100 | 125 | `displayPreferencePane:` | Resolve pane dari id toolbar, fire will-show, resize window |
| 135 | 158 | `changeDefaultFont:` | Terapkan font panel ke pref general atau editor sesuai `fontChangeTarget` |
| 166 | 243 | `setupToolbar` | Buat semua toolbar item, wire selector, pasang pane default |
| 248 | 258 | `_resizeWindowForContentView:` | Resize window sekitar content view baru |
| 267 | 275 | `windowWillClose:` | Tutup color panel + akhiri editing first-responder (flush save) |
| 280 | 308 | `toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:` | Kembalikan toolbar item prebuilt, else stub |
| 310 | 322 | `toolbarAllowedItemIdentifiers:` | Array id toolbar diizinkan |
| 324 | 336 | `toolbarDefaultItemIdentifiers:` | Urutan default id toolbar |
| 338 | 350 | `toolbarSelectableItemIdentifiers:` | Id toolbar selectable |

### Temuan / UNKNOWN
- **`shortcutItem` nil laten** — dirujuk PC:298 untuk `SPPreferenceToolbarShortcuts`, tapi tak ada shortcut item dialokasikan di `setupToolbar` (166–243) dan tak ada pane shortcuts di `preferencePanes` (85–93). `SPPreferenceToolbarShortcuts` tetap muncul di allowed/default/selectable (317, 331, 345). **UNKNOWN** apakah pane Shortcuts sengaja dihapus.
- `mcpPreferencePane` dibuat di `init` (65) karena pane MCP tanpa XIB; pane lain outlet XIB.
- `appearancePopUp`/`appearanceMacOSVersionLabel` di-wire XIB; disabled di bawah macOS 10.14 + label versi (78–83).
- Dispatch font: general → `[NSUserDefaults saveFont:]` + post `SPFontChangedNotification` (143–148); editor → `SPCustomQueryEditorFont` (151, 153).
- `displayPreferencePane:` match pane by `preferencePaneIdentifier` == toolbar `itemIdentifier` (110); min size window 500×450 (117).
- Pref key dimiliki: **tidak ada**.

---

## 2. `SPNetworkPreferencePane.m` (NET)

Pane Network — custom SSH client path, bookmark ssh/known-hosts config, dan daftar SSL cipher drag-reorder.

### Ivar/konstanta
Static `SPSSLCipherListMarkerItem = @"--"` NET:39. Kategori privat `SPMySQLConnection` mendeklarasikan `defaultSSLCipherList`, `legacySSLCipherList`, `_mergedSSLCipherPreferenceListFromSavedCipherString:disabledMarker:` NET:41–45 (**UNKNOWN** implementasinya). Class extension 47–54. `@synthesize` 58–62. Header ivar `.h:44–55`: outlet `sshClientPickerView`, `sshClientPath`, `sshConfigChooser`, `hiddenFileView`, `sslCipherView`; `@private _currentFilePanel`, `sslCiphers`, `sshConfigFile`, `knownHostsFile`. Property `.h:57–62`.

### `#pragma mark`
`91/92 (Preference pane protocol methods), 134/135 (Custom SSH client methods), 193/194 (PopUp Button), 539/540 (chooseSSHConfig), 645/646 (SSL cipher list methods), 697/698 (tableView Delegate)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 64 | 77 | `init` | Alokasi array + registrasi `SPBookmarksChangedNotification` |
| 79 | 83 | `dealloc` | Hapus observer bookmarks |
| 85 | 89 | `_refreshBookmarks` | Copy bookmark manager ke array lokal |
| 94–131 | | 5 method protocol pane (`preferencePaneView`/`Icon`/`Name`/`Identifier`/`ToolTip`/`WillBeShown`) | Identitas pane + refresh popup/cipher/drag type |
| 137 | 145 | `observeValueForKeyPath:ofObject:change:context:` | KVO hidden-file panel |
| 147 | 150 | `updateHiddenFiles` | Terapkan pref hidden files |
| 152–191 | | `pickSSHClientViaFileBrowser:` / `pickSSHClient:` | Pilih/persist client SSH kustom |
| 196–289 | | `updateKnownHostsConfig:` / `updateSSHConfig:` | Tangani pilihan popup config + save |
| 291 | 437 | `updateSSHConfigPopUp:` | Bangun ulang popup config dari bookmark; validasi known-hosts |
| 439 | 520 | `checkSSHConfigFileForUserKnownHostsFile:` | Parse config + include, validasi UserKnownHostsFile |
| 522 | 537 | `checkUserKnownHostsFilesAreWritable` | Pindah file known-hosts tak resolve ke error list |
| 542 | 642 | `chooseSSHConfigWithOptions:` | File panel, bookmark read/read-write, save path |
| 648–695 | | `loadSSLCiphers` / `storeSSLCiphers` / `resetCipherList:` / `defaultSSLCipherList` | Merge/save/reset daftar cipher |
| 700–774 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / `isGroupRow:` / `shouldSelectRow:` / `acceptDrop:row:dropOperation:` / `validateDrop:` / `pasteboardWriterForRow:` | Tabel cipher drag-reorder |

### Temuan / UNKNOWN
- Pref key: `SPHiddenKeyFileVisibilityKey`, `SPSSHClientPath` (`SPConstants.m:287`), `SPSSHConfigContainsUserKnownHostsFile`, `SPSSHUsualKnownHostsFile`, `SPSSHConfigFile` (`:94`), `SPSSHDefaultKnownHostsFile`, `SPSSLCipherListKey` (`:288`).
- ⚠️ **Bug dealloc:** `dealloc` memanggil `[self removeObserver:self forKeyPath:SPBookmarksChangedNotification]` (82) — API KVO dipakai untuk registrasi `NSNotificationCenter` (73); observer **tak pernah** dilepas dari notification center.
- Konvensi tag popup: `sshConfigChooser` "tag @1" (336–337), `knownHostsChooser` "tag @2" (354); tag menggerakkan cabang validasi (300, 337, 355). **UNKNOWN** apakah tag benar-benar diset di XIB.
- KVO `SPHiddenKeyFileVisibilityKey` ditambah sebelum tiap file panel (162–165, 564–567), dilepas di completion (170). `user_defaults_set_bool_ud` (`SPConstants.h:755`) dipakai 207, 384, 433.
- Bookmark stale diserahkan ke pane Files via `SPPreferenceController displayPreferencePane:fileItem` (421–423).
- `sshConfigFile`/`knownHostsFile` header ivar (`.h:53–54`) **UNKNOWN** — tak dirujuk di file ini.

---

## 3. `SPFilePreferencePane.m` (FILE)

Pane Files — daftar bookmark sandbox yang diberi, tandai stale, jalankan flow (re)grant file panel.

### Ivar
Class extension 36–45 (property writeback `bookmarks`/`staleBookmarks`/`selectedRows`/`weHaveStaleBookmarks`/`userClickedCancel`). `@synthesize` 49. Header ivar `.h:41–46`: `fileView`, `hiddenFileView`; `@private _currentFilePanel`, `fileNames`. Property `.h:50–51` `staleLabel`, `revokeButton`. **Tak ada konstanta file-local.**

### `#pragma mark`
`338/339 (File operations), 559/560 (Granted Files View)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 51 | 67 | `init` | Alokasi array + registrasi observer bookmarks |
| 69 | 178 | `doubleClick:` | Simulasi flow stale-file re-grant dari double-click |
| 180 | 183 | `dealloc` | Log teardown saja |
| 185 | 190 | `_refreshBookmarks` | Handler notifikasi reload list |
| 192–214 | | `preferencePaneIcon`/`Identifier`/`Name`/`ToolTip`/`View` | Identitas pane |
| 220 | 229 | `preferencePaneWillBeShown` | Load bookmark + re-grant stale bila perlu |
| 231 | 336 | `loadBookmarks` | Merge stale setelah granted; reload tabel |
| 341 | 362 | `revokeBookmark:` | Revoke bookmark terpilih + reload |
| 364 | 381 | `addBookmark:` | Bangun option panel tambah file/folder granted |
| 383–396 | | `observeValueForKeyPath:…` / `updateHiddenFiles` | KVO hidden-file |
| 399 | 556 | `chooseFileWithOptions:` | File panel, buat bookmark, rekursi multi-select |
| 565–599 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / `willDisplayCell:…` / `tableViewSelectionDidChange:` | Tabel granted files |

### Temuan / UNKNOWN
- **`tableViewSelectionDidChange:` ditulis `-(void)`** tanpa spasi (596) — tak match anchor `^[-+] (`.
- Pref key: hanya `SPHiddenKeyFileVisibilityKey` (385, 395, 440). Pane ini tak punya pref unik — file grant hidup di `SecureBookmarkManager.sharedInstance` (235–236, 350–351).
- ⚠️ **`dealloc` (180–183) hanya log**; tidak melepas observer `SPBookmarksChangedNotification` (registrasi 63) → leak observer.
- Aritmetika index stale: `fileNames` = granted bookmarks lalu stale ditambahkan (255–270, 320–327).
- Rekursi multi-select: `chooseFileWithOptions:` panggil ulang dirinya selama `options.fileNames` sisa (542–545).
- Known-hosts upgrade: URL di `knownHostsBookmarks` → option dipaksa read-write (501–505).

---

## 4. `SPGeneralPreferencePane.m` (GEN)

Pane General — font hasil global, popup default favorite, konstruksi menu favorites.

### Ivar/konstanta
Static `SPDatabaseImage = @"database-small"` GEN:41. Class extension 43–47. Header ivar `.h:42–45`: `folderImage`; outlet `globalResultFontName`, `defaultFavoritePopup`.

### `#pragma mark`
`51/52 (Initialisation), 64/65 (IB action methods), 122/123 (Public API), 167/168 (Private API), 220/221 (Preference pane protocol methods)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 54 | 62 | `awakeFromNib` | Bangun ikon folder 16pt untuk menu group |
| 70 | 83 | `updateDefaultFavorite:` | Simpan index/judul default-favorite ke pref |
| 85 | 105 | `showGlobalResultFontPanel:` | Set font target, tampilkan font panel + tombol system-font |
| 107 | 120 | `changeDefaultFont:` | Terapkan system font bila accessory ditekan, lalu forward |
| 128 | 130 | `updateDisplayedFontName` | Terapkan font tersimpan ke field sampel |
| 135 | 165 | `updateDefaultFavoritePopup` | Bangun ulang popup dari tree favorites |
| 177 | 218 | `_constructMenuItemsForNode:atLevel:` | Bangun menu/submenu favorites rekursif |
| 223–256 | | `preferencePaneView`/`Icon`/`Name`/`Identifier`/`ToolTip`/`WillBeShown` | Identitas pane + refresh |

### Temuan / UNKNOWN
- Pref key: `SPSelectLastFavoriteUsed` (80, 164), `SPDefaultFavorite` (82, 164) — pane ini satu-satunya penulis (`SPConstants.m` **UNKNOWN** baris).
- Key dict node: `SPFavoriteNameKey` (207), `SPFavoriteIDKey` (209).
- **Wiring lintas-controller:** perubahan font dirutekan lewat window delegate yang di-cast ke `SPPreferenceController` (`setFontChangeTarget:` 86, `changeDefaultFont:` 119). Bergantung window delegate = `SPPreferenceController` (**UNKNOWN** bila selalu benar).
- Accessory system-font: tombol inline "Use System Font" (95–101) tag `1001`; `changeDefaultFont:` cek `tag == 1001` (108) + set panel font (113–116).
- Akses tree: `favoritesTree.childNodes[0].childNodes` (142); kosong → item disabled "No Favorties" (158–160, typo di source).

---

## 5. `SPMCPPreferencePane.swift` (MCP)

Pane preferensi server MCP — UI programatik (tanpa XIB): enable/disable server, toggle read-only, set port, pilih folder export, tampilkan endpoint.

### Ivar/konstanta
`kMCPDefaultPort = 8765`, `kMCPMinPort = 1024`, `kMCPMaxPort = 65535` MCP:31–33. `@objc(SPMCPPreferencePane) final class SPMCPPreferencePane: SPPreferencePane, SPPreferencePaneProtocol, NSTextFieldDelegate` (38–39). Property 46–51: `enableCheckbox`, `readOnlyCheckbox`, `portField`, `statusLabel`, `exportPathField`, `endpointField`. Computed: `defaults` (44), `endpointURLString` (346–348), `currentPort` (350–353), `currentExportPath` (355–359).

### `// MARK:`
`53 (View lifecycle), 189 (SPPreferencePaneProtocol), 225 (Actions), 322 (Private helpers), 361 (Label factory helpers)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 55 | 187 | `loadView` | Bangun seluruh layout pane frame-based; wire kontrol + constraint |
| 191–213 | | `preferencePaneView`/`Icon`/`Name`/`Identifier`/`ToolTip` | Identitas pane |
| 215 | 223 | `preferencePaneWillBeShown` | Paksa view load lalu refresh status + endpoint |
| 228 | 240 | `toggleMCPServer:` | Persist flag enabled; refresh status sekarang + setelah 0.3 s |
| 243 | 245 | `toggleReadOnly:` | Persist flag read-only |
| 248 | 277 | `updatePort:` | Validasi digit/range port, else alert; persist + refresh |
| 280 | 314 | `chooseExportPath:` | Directory panel, simpan path + bookmark security-scoped |
| 316 | 320 | `copyEndpoint:` | Copy URL endpoint ke pasteboard |
| 324 | 340 | `refreshStatus` | Set teks/warna status dari enabled + server running |
| 342 | 344 | `refreshEndpoint` | Tulis URL endpoint ke field read-only |
| 363–376 | | `makeLabel` / `makeSmallLabel` | Factory label |

### Temuan / UNKNOWN
- Pref key: `SPMCPServerEnabled` (`SPConstants.m:338`), `SPMCPReadOnly` (`:341`), `SPMCPServerPort` (~`:339`), `SPMCPExportPath` (`:340`). Pane ini satu-satunya UI owner keempatnya; dibaca `SPAppController+MCP.swift` + `SPMCPServer.swift`.
- **Tanpa XIB**: `loadView` bangun seluruh layout (55–187); constraint height eksplisit karena pane frame-based melaporkan `fittingSize == 0` (61–68).
- Guard lazy-load: `preferencePaneWillBeShown` menyentuh `view` untuk memaksa `loadView` sebelum `refreshStatus()` (216–221).
- Koordinasi server (komentar): `SPAppController` observe perubahan defaults + start/stop server (232–235, 268–271); status baca `SPMCPServer.shared.isRunning` (325).
- Sandbox export: bookmark security-scoped via `SecureBookmarkManager.sharedInstance.addBookmarkFor(...)` (297–304).

---

## Ringkasan lintas-file
- **Registry/toolbar** terpusat di PC:85–93 (array pane) + PC:312–349 (id toolbar, termasuk `SPPreferenceToolbarShortcuts` nyasar).
- **Base bersama** `SPPreferencePane` menyediakan `prefs` + `modifyAndReturnBookmarkHelpView`.
- **Asimetri teardown observer:** NET + FILE sama-sama registrasi `SPBookmarksChangedNotification` (NET:73, FILE:63); `dealloc` NET pakai API KVO (82), `dealloc` FILE tak melepas apa pun (180–183) — keduanya SUSPECT.
- **Konstanta pref** semua di `Source/Other/Data/SPConstants.m`.
