# Audit: `SPBundleManager.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/Other/SPBundleManager.m`. Baseline commit `02c910932`.

- File: 1119 baris (`@implementation` 53, `@end` 1119); header 51 baris.
- Peran: singleton manager yang menemukan/memuat/migrasi command plugin
  berurutan ("bundle"), mendaftarkan scope/trigger/key-equivalent, dan
  meng-dispatch command bundle ke app atau responder terfokus.
- Total: 19 definisi method.

Catatan bridging: `loadBundleAt:error:` **tidak** didefinisikan di file ini — itu
method throwing ObjC-bridged dari `SPBundleManagerAdditions.swift:33` (`loadBundle(at:) throws`).
`SABundleVersionUpdater` adalah helper Swift (`SABundleVersionUpdater.swift:8`).

## 1. Inventaris ivar/property

### Property publik `.h:16–24`
`alreadyBeeped`, `bundleItems`, `bundleCategories`, `bundleTriggers`, `bundleUsedScopes`,
`bundleHTMLOutputController`, `bundleKeyEquivalents`, `installedBundleUUIDs`, `foundInstalledBundles`.

### Property class-extension `.m:43–50`
`bundleEditorController` (`SPBundleEditorController*`) (45), `fileManager` (46),
`migratedLegacyBundles` (47), `badBundles` (48).

### `@synthesize` `.m:55–59`
`fileManager`, `bundleEditorController` (55); `alreadyBeeped` (56); `badBundles`, `foundInstalledBundles` (57);
`migratedLegacyBundles` (58); enam registry (59). Static file-local: `static SPBundleManager *sharedManager = nil;` (61).

Extern (SPConstants.m): `SPBundleAppSchema=@"sequelace"` (459), `SPBundleLegacyAppSchema=@"sequelpro"` (460),
`SPBundleCurrentVersion=2` (464), `SPUserBundleFileExtension=@"spBundle"`/`...V2=@"saBundle"` (79–80),
`SPBundleSupportFolder=@"Bundles"` (101), `SPBundleFileName=@"command.plist"` (466),
`SPBundleVersionKey=@"bundleVersion"` (461), `SPBundleDeletedDefaultBundlesKey=@"deletedDefaultBundles"` (282).

## 2. Semua `#pragma mark`

`.m:100 (legacy string methods), 184 (legacy bundle rename)`. Hanya dua.

## 3. Tabel method (19 definisi, urut baris)

### Singleton & init (53–98)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 63 | 74 | `+shared` | Singleton lazy main-thread via `dispatch_once_on_main_thread` |
| 76 | 98 | `init` | Init fileManager + semua ivar collection |

### legacy string methods (100)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 101 | 154 | `findLegacyStrings:` | Scan file bundle untuk schema `sequelpro`, kembalikan map pengganti |
| 156 | 182 | `replaceLegacyString:` | Tulis ulang isi (schema → `sequelace`) ke tiap file |

### legacy bundle rename + sisa (184)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 185 | 297 | `renameLegacyBundles` | Migrasi `.spBundle`→`.saBundle`, naikkan versi, perbaiki string legacy |
| 299 | 309 | `doOrDoNotBeep:` | Beep sekali per key unik, di-throttle `alreadyBeeped` |
| 311 | 335 | `removeBundle:` | Hapus folder bundle di Application Support, catat bad bundle |
| 337 | 341 | `openBundleEditor:` | Buat+tampilkan window bundle editor (lazy) |
| 343 | 718 | `reloadBundles:` | Rebuild penuh: load bundle, migrasi, update default, rebuild menu |
| 724 | 799 | `bundleCommandDispatcher:` | Resolve scope key-equivalent + rutekan command ke first responder |
| 801 | 952 | `executeBundleItemForApp:` | Jalankan bash command bundle, petakan exit-code/redirect ke output action |
| 954 | 1080 | `openUserBundleAtPath:` | Instal bundle user, tangani legacy string, UUID, duplikat |
| 1083 | 1086 | `validateMenuItem:` | Selalu YES; komentar bilang mungkin tak dipanggil |
| 1089 | 1092 | `bundleCategoriesForScope:` | Daftar kategori terdaftar untuk scope |
| 1094 | 1097 | `bundleCommandsForTrigger:` | Command string terdaftar untuk trigger |
| 1099 | 1102 | `bundleItemsForScope:` | Item menu bundle terdaftar untuk scope |
| 1104 | 1107 | `bundleKeyEquivalentsForScope:` | Map key-equivalent → item untuk scope |
| 1109 | 1112 | `addHTMLOutputController:` | Append controller window output HTML ke array tracking |
| 1114 | 1117 | `removeHTMLOutputController:` | Hapus controller window output HTML dari tracking |

## 4. Temuan menonjol / UNKNOWN

**`reloadBundles:` (343–718) — rutin inti**
- **Search path dua-pass:** `bundlePaths` = `[0]` Application Support `Bundles` (buat bila perlu), `[1]` app-bundle `Default Bundles` (368–371); `processDefaultBundles` di-flip YES setelah pass pertama (697).
- **Cleanup di entry:** tutup window output HTML tersembunyi (346–350), reset `foundInstalledBundles` (352), bersihkan enam registry (356–361).
- **Filter ekstensi:** hanya `.spBundle`/`.saBundle` (case-insensitive) (403).
- **Load gagal ⇒ self-heal:** bundle rusak di-beep + `removeBundle:` menghapusnya (413–419).
- **Tail:** `renameLegacyBundles` (701), bersihkan pref `doBundleUpdate` bila diset (703–705), dan bila default bundle diupdate — alert + **recurse** `reloadBundles:nil` (708–712); else `[SPAppDelegate rebuildMenus]` (715).

**Versioning default bundle (429–581)**
- Hanya berlaku default bundle ber-UUID (431) dan hanya pass 1 (`processDefaultBundles` 433). **Deleted-default suppression:** bundle di daftar pref `deletedDefaultBundles` di-skip (436–445).
- **Perbandingan versi** delegasi ke `SABundleVersionUpdater shouldUpdateDefaultBundleWithInstalledVersion:bundledVersion:` (465–466); Swift memperlakukan versi hilang sebagai 0 dan return `bundled > installed` (`SABundleVersionUpdater.swift:11-15`).
- **Default dimodifikasi user** (key `defaultBundleWasModified`) diduplikasi dengan UUID baru + sufiks nama `" (user)"` dan `isDefaultBundle` dilepas (490–544); bila belum dimodifikasi, salinan lama dihapus (545–557). Gate update: `doBundleUpdate || installedBundleInfo == nil || needsVersionUpdate` (477).
- **UNKNOWN/smell:** di 414 log memakai `error.localizedDescription` enumeration direktori luar, bukan `loadErr`; di 263 pesan bilang "Could not delete" padahal itu write; cabang `writeToFile:atomically:` (270) tanpa logging error.

**Trigger (611–619)**: `bundleTriggers[trigger]` menyimpan string `"<infoPath>|<scope>|<uuid-or-empty>"`, field ketiga UUID hanya bila output `showAsHTML` (614–618). **UNKNOWN:** tak ada escaping bila `infoPath` memuat `|`.

**Key equivalents (621–656)**: parse char terakhir sebagai key, char prefiks sebagai modifier `^`=Control, `@`=Command, `~`=Option, `$`=Shift (627–631). Dedup entry sebelumnya untuk UUID sama (641–645), simpan `{path,title,tooltip,uuid}` di `bundleKeyEquivalents[scope][keyEquivalent]` (646–653). **Dispatch** (724–799): scope dari selector firstResponder (`executeBundleItemForInputField:`/`ForDataTable:`) else General (737–743); fallback General, beep bila tak ada (749–758); bila >1 match → `SPChooseMenuItemDialog` (771–773, idx −1 = cancel).

**Migrasi dari `spBundle` (185–297)**: rename `.spBundle`→`.saBundle` via `dropSuffixWithSuffix:` + append (204–207); skip path sudah migrasi (194–197). Bila target sudah ada, hapus folder legacy (216–225); `moveItemAtPath:` gagal beep + return (229–233). Verifikasi pasca-move reload `command.plist`, set `bundleVersion = 2`, tulis ulang, update `bundle["path"]` (238–271); migrasi invalid dihapus via `removeBundle:` (243–249). Setelah migrasi, string `sequelpro` ditemukan + diganti (284–296).

**`openUserBundleAtPath:` (954–1080)**: pastikan dir Application Support `Bundles` ada (957–967); bila string legacy ditemukan, prompt Install/Cancel (1006–1027). Wajib UUID atau warn + abort (1030–1033); lazy-init dengan `reloadBundles:` bila `installedBundleUUIDs` kosong (1036–1038). Bila UUID sudah terinstal, tawarkan Update → hapus path lama (1040–1055); pindahkan bundle, update window editor + menu (1057–1076); else error "already exists" (1077–1079).

**`executeBundleItemForApp:` (801–952)**: resolve item via menu tag-1000000 atau `toolTip` (805–814); bangun env (`SPBundleShellVariable*`) + file input temp (839–852); jalankan via `SPBundleCommandRunner runBashCommand:` (860–868). Petakan exit code (`SPBundleRedirectAction*`) ke output action bila `err` diset (877–910); `err.code == 9` menekan alert (946). Output: text/HTML tooltip, atau reuse/create `SABundleHTMLOutputWindowController` by UUID (917–944).

**Meta**: `validateMenuItem:` no-op YES dengan komentar "dont think this is called" (1082–1086).
**UNKNOWN:** pemanggil `addHTMLOutputController:`/`removeHTMLOutputController:` di luar file ini.
