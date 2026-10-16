# Audit: `SPDatabaseDocument.m`

Audit method-by-method file raksasa. Setiap definisi method dicantumkan sekali
dengan rentang baris + tujuan satu baris. `Start`/`End` adalah nomor baris di
`Source/Controllers/MainViewControllers/SPDatabaseDocument.m`. Baseline commit
`02c910932`.

- File `.m`: 6436 baris; header `.h`: 483 baris.
- Peran: controller pusat untuk satu tab/jendela koneksi database — memiliki
  koneksi MySQL, seleksi database/tabel, switching view (tab), orkestrasi
  task-progress, save/restore sesi `.spf`, scripting URL-scheme, dan printing.
- Struktur: **tidak ada** `@implementation X (Category)`; hanya satu
  `@implementation SPDatabaseDocument` (baris 139) dan satu class extension
  `@interface SPDatabaseDocument ()` (baris 98–137).
- Total: 230 definisi method (1 class method, 229 instance method).

## 1. Inventaris ivar/property

### Header `@interface SPDatabaseDocument : NSObject <…>` (`.h`)
| Baris | Isi |
| --- | --- |
| 70 | Deklarasi `@interface` + conformances (`SPConnectionControllerDelegateProtocol, SPMySQLConnectionDelegate, NSTextFieldDelegate, NSToolbarDelegate, SPCountedObject`) |
| 71–224 | Blok ivar |
| 72–150 | IBOutlet: sub-controller tab (73–85), accessory view status (87–89), `userManagerInstance`/`serverSupport` (91–92), filter/scroll/`parentView` (94–99), sheet DB & charset helper (101–118), field copy/rename (120–127), `chooseDatabaseButton`/`historyControl`/`tableTabView` (129–131), info table & splitter (133–135), `encodingPopUp` (137), `customQueryTextView` (139), `dbTablesTableView` (141), view create-syntax (143–145), password input window + `passwordSheetReturnCode` (147–151) |
| 153–224 | State non-outlet: `mySQLConnection` (154); controller/`selectedTableName`/`selectedTableType` (156–161); flag load `structureLoaded…allowSplitViewResizing` (163–169); `selectedDatabase`/`mySQLVersion`/`selectedDatabaseEncoding`/`prefs`/`undoManager` (171–175); `selectEncodingMenu`, `_supportsEncoding`, `_isConnected`, `_isWorkingLevel`, `_mainNibLoaded`, `databaseListIsSelectable`, `_queryMode`, `_isSavedInBundle`, `_workingTimeout` (177–186); `taskController` (188); `chooseDatabaseToolbarItem` (190); `printRenderer` (192); `allDatabases`/`allSystemDatabases` (194–195); `queryEditorInitString` (197); penyimpanan file/sesi (`sqlFileURL`…`spfDocData`, 199–204); `runningActivitiesArray` (206); `printThread` (208); `statusValues` (210); return code alert (213–214); `@property isProcessing`…`instanceId` (217–223) |
| 226–255 | `@property` (outlet accessory save-connection 228–235; `dbTablesTableView`, `sqlFileURL`, `sqlFileEncoding`, `isProcessing`, `processID`, `mainToolbar` 237–242; `parentWindowController`…`multipleLineEditingButton` 244–255) |
| 257–481 | Deklarasi method publik, dikelompokkan `#pragma mark` |

### Class extension `.m` `@interface SPDatabaseDocument ()` — 98–137
| Baris | Isi |
| --- | --- |
| 98 | Conformance `<SADatabaseSelectionDelegate, NSToolbarItemValidation>` |
| 100 | `allowSplitViewResizing` dideklarasikan ulang read/write |
| 103–112 | Property privat: `hideConsoleImage`, `showConsoleImage`, `textAndCommandMacwindowImage`, `parentWindowController`, `appIsTerminating`, `preferenceObserversRegistered`, `mainToolbar` |
| 114–122 | Deklarasi method privat (`_addDatabase`…`_removePreferenceObservers`) |
| 124–127 | `#pragma mark - SPDatabaseViewControllerPrivateAPI` |
| 129–135 | `#pragma mark - SPConnectionDelegate` |

Blok `@synthesize` 141–159. Static file-scope (90–94): `SPNewDatabaseDetails`,
`SPNewDatabaseName`, `SPNewDatabaseCopyContent`, `SPDatabaseDocumentInstanceCounter`.

## 2. Semua `#pragma mark` (urut baris)

`124, 129, 161, 328, 340, 402, 403 (Connection callback and methods), 590, 591 (Database methods), 1044, 1045 (Console methods), 1092, 1093 (Navigator methods), 1109, 1110 (Task progress and notification methods), 1281 (SATaskControllerDelegate), 1300, 1301 (Encoding Methods), 1481, 1482 (Table Methods), 2026, 2027 (Other Methods), 2197, 2198 (Accessor methods), 2416, 2417 (Notification center methods), 2460, 2461 (Menu methods), 3159, 3160 (Titlebar Methods), 3209, 3210 (Toolbar Methods), 3433, 3434 (Tab methods), 3486, 3487 (NSDocument compatibility), 3557, 3558 (State saving and setting), 4137, 4138 (Connection controller delegate methods), 4162, 4163 (Scheme scripting methods), 4772, 4773 (Text field delegate methods), 4796, 4797 (General sheet delegate methods), 4820, 4821 (SplitView delegate methods), 4840, 4841 (Datasource methods), 4883, 4884 (Status accessory view), 4918, 4919 (Private API), 5166 (SADatabaseSelectionDelegate), 5377 (SPDatabaseViewController), 5379 (Getters), 5430, 5431 (Tab view control and delegate methods), 5598, 5599 (Table control), 5899 (SPMySQLConnection delegate methods), 6064 (SPPrintController), 6423`.

## 3. Tabel method (230 definisi, urut baris)

### `@implementation` pre-pragma (161–327)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 163 | 164 | `+initialize` | Class initializer kosong |
| 166 | 248 | `initWithWindowController:` | Init state, image, task controller; muat nib DBView |
| 250 | 326 | `awakeFromNib` | Wiring sekali: toolbar, connection controller, observer, sheet |

### Accessors (328)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 330 | 338 | `mainToolbar` | Buat/cache NSToolbar utama secara lazy |
| 345 | 352 | `closePasswordSheet:` | Tutup sheet password enkripsi, set return code |
| 357 | 366 | `backForwardInHistory:` | Rutekan menu history by tag ke back/forward |
| 371 | 383 | `goBackInHistory` | Commit edit lalu mundur di history |
| 388 | 400 | `goForwardInHistory` | Commit edit lalu maju di history |

### Connection callback and methods (403)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 409 | 578 | `setConnection:` | Pasang koneksi; konfigurasi semua controller, restore sesi |
| 585 | 588 | `getConnection` | Kembalikan koneksi MySQL master |

### Database methods (591)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 598 | 600 | `setDatabases:` | Wrapper IBAction untuk `-setDatabases` |
| 602 | 621 | `setDatabases` | Bangun ulang popup DB; persist `allDatabases`/`allSystemDatabases` |
| 627 | 647 | `chooseDatabase:` | Tangani pilihan popup DB; commit/select |
| 652 | 678 | `selectDatabase:item:` | Mulai task + dispatch seleksi DB/tabel ke thread |
| 683 | 721 | `addDatabase:` | Tampilkan sheet tambah DB; buat DB saat OK |
| 726 | 750 | `alterDatabase` | Tampilkan sheet ALTER DATABASE dengan charset aktif |
| 752 | 815 | `compareDatabase:` | Prototipe banding DB eksperimental (NSLog saja) |
| 820 | 848 | `copyDatabase` | Peringatkan objek non-tabel; tampilkan sheet copy DB |
| 853 | 868 | `renameDatabase` | Tampilkan sheet rename DB terpilih |
| 873 | 888 | `removeDatabase:` | Konfirmasi lalu drop database aktif |
| 893 | 895 | `refreshTables` | Minta SPTablesList refresh daftar tabel |
| 900 | 908 | `showServerVariables` | Tampilkan sheet server-variables (lazy) |
| 913 | 921 | `showServerProcesses` | Tampilkan window process-list (lazy) |
| 923 | 931 | `shutdownServer` | Konfirmasi + shutdown server MySQL |
| 936 | 939 | `allDatabaseNames` | Kembalikan array nama database user |
| 944 | 947 | `allSystemDatabaseNames` | Kembalikan array nama database sistem |
| 953 | 956 | `showErrorSheetWith:` | Tampilkan warning sheet dari array [title,message] |
| 963 | 983 | `refreshCurrentDatabase` | Query ulang `SELECT DATABASE()` + sinkron seleksi |
| 992 | 1006 | `setCurrentDatabaseFromQueryContext:` | Commit konteks DB yang sudah diturunkan dari query |
| 1008 | 1011 | `navigatorSchemaPathExistsForDatabase:` | Tanya navigator apakah schema path ada |
| 1013 | 1016 | `getDbStructure` | Kembalikan dictionary struktur DB navigator |
| 1018 | 1021 | `allSchemaKeys` | Kembalikan schema key navigator |
| 1023 | 1042 | `showGotoDatabase` | Picker modal "go to database[.table]" |

### Console methods (1045)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1050 | 1061 | `toggleConsole` | Show/hide window console query |
| 1066 | 1075 | `showConsole` | Refresh entri lalu front window console |
| 1080 | 1082 | `clearConsole:` | Bersihkan pesan console via SPQueryController |
| 1087 | 1090 | `setQueryMode:` | Simpan query mode untuk keputusan logging |

### Navigator methods (1093)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1098 | 1107 | `toggleNavigator` | Show/hide navigator; refresh entri saat tampil |

### Task progress and notification methods (1110)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1125 | 1161 | `startTaskWithDescription:` | Naikkan working level; mulai UI task di level pertama |
| 1166 | 1169 | `setTaskDescription:` | Update teks deskripsi task |
| 1176 | 1179 | `setTaskPercentage:` | Teruskan progress determinate ke controller |
| 1188 | 1191 | `setTaskProgressToIndeterminateAfterDelay:` | Reset indikator ke indeterminate |
| 1196 | 1224 | `endTask` | Turunkan working level; akhiri UI saat semua task selesai |
| 1230 | 1239 | `enableTaskCancellationWithTitle:callbackObject:callbackFunction:` | Aktifkan tombol cancel dengan callback |
| 1244 | 1253 | `disableTaskCancellation` | Nonaktifkan cancellation (no-op bila idle) |
| 1259 | 1262 | `isWorking` | YES bila ada task level aktif |
| 1267 | 1270 | `setDatabaseListIsSelectable:` | Set flag selectability daftar database |
| 1276 | 1279 | `setTaskIndicatorShouldAnimate:` | Pause/restart animasi indikator task |

### SATaskControllerDelegate (1281)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1283 | 1286 | `taskParentWindow` | Kembalikan parent window untuk UI task |
| 1288 | 1298 | `taskControllerDidRequestCancellation` | Kill/cancel query yang jalan saat permintaan cancel |

### Encoding Methods (1301)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1306 | 1344 | `setConnectionEncoding:reloadingViews:` | Set charset koneksi (transport utf8-latin1); reload view |
| 1351 | 1363 | `updateEncodingMenuWithSelectedEncoding:` | Centang item menu encoding yang cocok |
| 1368 | 1397 | `encodingTagFromMySQLEncoding:` | Map string charset MySQL → tag menu |
| 1402 | 1430 | `mysqlEncodingFromEncodingTag:` | Map tag menu → string charset MySQL |
| 1436 | 1439 | `databaseEncoding` | Kembalikan encoding DB ter-cache (default latin1) |
| 1445 | 1464 | `detectDatabaseEncoding` | Query charset default DB; fallback latin1 |
| 1469 | 1471 | `chooseEncoding:` | Terapkan encoding dari item menu |
| 1476 | 1479 | `supportsEncoding` | Apakah pemilihan charset didukung |

### Table Methods (1482)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1487 | 1592 | `showCreateTableSyntax:` | Ambil/copy/tampilkan CREATE syntax item terpilih |
| 1597 | 1601 | `copyCreateTableSyntax:` | Copy CREATE syntax ke clipboard |
| 1606 | 1659 | `checkTable` | Jalankan CHECK TABLE + sheet hasil |
| 1664 | 1717 | `analyzeTable` | Jalankan ANALYZE TABLE + sheet hasil |
| 1722 | 1776 | `optimizeTable` | Jalankan OPTIMIZE TABLE + sheet hasil |
| 1781 | 1834 | `repairTable` | Jalankan REPAIR TABLE + sheet hasil |
| 1839 | 1893 | `flushTable` | Jalankan FLUSH TABLE + sheet hasil |
| 1898 | 1934 | `checksumTable` | Jalankan CHECKSUM TABLE + sheet hasil |
| 1939 | 1961 | `saveCreateSyntax:` | Simpan text view create-syntax ke file .sql |
| 1966 | 1980 | `copyCreateTableSyntaxFromSheet:` | Copy teks sheet create-syntax ke pasteboard |
| 1985 | 1989 | `focusOnTableContentFilter` | Pindah ke content view + fokus filter |
| 1994 | 1998 | `showFilterTable` | Pindah ke content view + toggle rule editor |
| 2003 | 2006 | `performFindPanelAction:` | Fokus filter content saat Cmd-F |
| 2012 | 2015 | `exportSelectedTablesAs:` | Export tabel terpilih dalam format dipilih |
| 2020 | 2024 | `exportData` | Buka dialog export data bila terhubung |

### Other Methods (2027)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2029 | 2032 | `multipleLineEditingButtonClicked:` | Persist pref "edit in sheet" dari state tombol |
| 2039 | 2042 | `initQueryEditorWithString:` | Simpan query untuk disisip setelah koneksi |
| 2048 | 2051 | `closeSheet:` | Stop modal dengan code 0 |
| 2056 | 2060 | `closePanelSheet:` | Akhiri sheet + order out window |
| 2065 | 2087 | `showUserManager` | Buka user manager (setelah cek privilege) |
| 2092 | 2095 | `doPerformQueryService:` | Jalankan query di custom query editor |
| 2100 | 2103 | `doPerformLoadQueryService:` | Muat teks query ke custom query editor |
| 2108 | 2118 | `flushPrivileges` | Jalankan FLUSH PRIVILEGES + lapor hasil |
| 2124 | 2130 | `connect` | Minta connection controller reconnect bila perlu |
| 2132 | 2146 | `closeConnection` | Disconnect koneksi master + struktur |
| 2151 | 2159 | `observeValueForKeyPath:ofObject:change:context:` | Terapkan pref console-logging/edit-in-sheet (KVO) |
| 2161 | 2164 | `helpViewerClient` | Kembalikan instance help viewer client |
| 2169 | 2172 | `isUntitled` | Apakah dokumen tanpa file URL |
| 2179 | 2195 | `couldCommitCurrentViewActions` | Akhiri editing; commit baris struktur/content |

### Accessor methods (2198)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2203 | 2212 | `host` | Kembalikan host koneksi (localhost untuk socket) |
| 2217 | 2228 | `name` | Kembalikan nama tampil koneksi |
| 2233 | 2277 | `connectionID` | Bangun string identitas unik per-koneksi |
| 2283 | 2314 | `tabTitleForTooltip` | Bangun tooltip judul window/tab penuh |
| 2319 | 2322 | `database` | Kembalikan nama database terpilih |
| 2327 | 2330 | `mySQLVersion` | Kembalikan string versi server ter-cache |
| 2335 | 2340 | `user` | Kembalikan user koneksi |
| 2345 | 2350 | `port` | Kembalikan port koneksi |
| 2352 | 2355 | `isSaveInBundle` | Kembalikan flag bundle-saved |
| 2357 | 2360 | `allTableNames` | Kembalikan semua nama tabel dari tables list |
| 2362 | 2372 | `createDatabaseInfo` | Bangun SPCreateDatabaseInfo untuk DB aktif |
| 2379 | 2414 | `currentlySelectedView` | Map item tab aktif ke enum SPTableViewType |

### Notification center methods (2417)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2422 | 2426 | `willPerformQuery:` | Tandai processing; mulai progress bar |
| 2431 | 2435 | `hasPerformedQuery:` | Bersihkan processing; stop progress bar |
| 2440 | 2458 | `applicationWillTerminate:` | Auto-save pref; toleransi tabel dirty; dump leak |

### Menu methods (2461)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2467 | 2568 | `saveConnectionSheet:` | Simpan query / koneksi (SPF) / sesi via save panel |
| 2572 | 2586 | `validateSaveConnectionAccessory:` | Enable/validasi kontrol accessory save-connection |
| 2588 | 2644 | `saveConnectionPanelDidEnd:returnCode:contextInfo:` | Tangani selesainya save panel (SQL/SPF/sesi) |
| 2646 | 2863 | `saveDocumentWithFilePath:inBackground:onlyPreferences:contextInfo:` | Serialisasi + tulis file .spf (opsi/enkripsi) |
| 2868 | 2887 | `openDatabaseInNewTab` | Duplikat tab dengan state saat ini, tabel dikosongkan |
| 2892 | 2894 | `importFile` | Teruskan ke objek import data |
| 2899 | 2901 | `importFromClipboard` | Teruskan import clipboard ke import data |
| 2907 | 2910 | `showMySQLHelp` | Buka TOC MySQL Help di help viewer |
| 2915 | 2918 | `makeTableListFilterHaveFocus:` | Fokus field filter tables list |
| 2921 | 2934 | `showConnectionDebugMessages:` | Tampilkan sheet pesan debug SSH tunnel |
| 2939 | 3126 | `validateMenuItem:` | Validasi/enable item menu per state |
| 3131 | 3141 | `addConnectionToFavorites` | Tambah koneksi aktif ke favorites |
| 3146 | 3149 | `isCustomQuerySelected` | YES bila item toolbar Custom Query aktif |
| 3154 | 3157 | `getCreateTableSyntaxWindow` | Kembalikan window create-syntax |

### Titlebar Methods (3160)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3170 | 3207 | `updateWindowTitle:` | Kumpulkan state; update judul/tab/tint via SAWindowTitleBuilder |

### Toolbar Methods (3210)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3215 | 3218 | `selectedToolbarItemIdentifier` | Kembalikan identifier item toolbar terpilih |
| 3223 | 3305 | `toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:` | Bangun item toolbar untuk identifier |
| 3307 | 3314 | `toolbarWillAddItem:` | Lacak item toolbar pemilih database |
| 3316 | 3323 | `toolbarDidRemoveItem:` | Bersihkan item pemilih database |
| 3328 | 3346 | `toolbarAllowedItemIdentifiers:` | Kembalikan identifier toolbar yang diizinkan |
| 3351 | 3368 | `toolbarDefaultItemIdentifiers:` | Kembalikan identifier toolbar default |
| 3373 | 3385 | `toolbarSelectableItemIdentifiers:` | Kembalikan identifier toolbar yang selectable |
| 3390 | 3431 | `validateToolbarItem:` | Enable/disable + re-image item toolbar |

### Tab methods (3434)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3439 | 3484 | `parentTabShouldClose` | Guard penutupan tab; auto-save, kill aktivitas, cleanup |

### NSDocument compatibility (3487)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3492 | 3502 | `setFileURL:` | Simpan URL .spf; update represented URL window |
| 3507 | 3510 | `fileURL` | Kembalikan salinan URL file .spf |
| 3516 | 3525 | `writeSafelyToURL:ofType:forSaveOperation:error:` | Alihkan save ke sheet save-connection |
| 3530 | 3538 | `isDocumentEdited` | Laporkan favorites/filter dokumen sebagai edit belum tersimpan |
| 3544 | 3550 | `displayName` | Judul window untuk preamble terputus |
| 3552 | 3555 | `undoManager` | Kembalikan undo manager dokumen |

### State saving and setting (3558)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3571 | 3730 | `stateIncludingDetails:` | Bangun dictionary state sesuai level detail diminta |
| 3732 | 3735 | `setState:` | Delegasi ke `setState:fromFile:` |
| 3742 | 3883 | `setState:fromFile:` | Terapkan state koneksi/sesi ke controller |
| 3889 | 4069 | `setStateFromConnectionFile:` | Baca/dekripsi file .spf + terapkan state-nya |
| 4074 | 4135 | `restoreSession` | Restore tabel, view, filter, seleksi dari sesi |

### Connection controller delegate methods (4138)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4143 | 4146 | `connectionControllerInitiatingConnection:` | Set judul window "Connecting…" |
| 4151 | 4155 | `connectionControllerConnectAttemptFailed:` | Reset judul window saat gagal |
| 4157 | 4160 | `connectionController` | Kembalikan connection controller |

### Scheme scripting methods (4163)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4168 | 4171 | `setTimeout` | Flag untuk memecah loop tunggu scheme-command |
| 4176 | 4662 | `handleSchemeCommand:` | Dispatch command URL-scheme (view/tabel/query/export) |
| 4664 | 4678 | `registerActivity` | Daftarkan aktivitas jalan; tampilkan pane aktivitas |
| 4680 | 4700 | `removeRegisteredActivity:` | Lepas aktivitas by pid; update pane |
| 4702 | 4712 | `setActivityPaneHidden:` | Toggle pane aktivitas vs scroll view table-info |
| 4714 | 4717 | `runningActivities` | Kembalikan array aktivitas jalan |
| 4719 | 4770 | `shellVariables` | Bangun dictionary env variabel shell bundle |

### Text field delegate methods (4773)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4778 | 4794 | `controlTextDidChange:` | Enable tombol add/copy/rename DB pada nama valid |

### General sheet delegate methods (4797)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4799 | 4818 | `window:willPositionSheet:usingRect:` | Posisi sheet "Reset Auto Increment" dekat baris |

### SplitView delegate methods (4821)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4823 | 4829 | `splitView:constrainMinCoordinate:ofSubviewAt:` | Paksa min 40px untuk divider pertama |
| 4831 | 4838 | `splitView:constrainMaxCoordinate:ofSubviewAt:` | Jaga min lebar 505px untuk pane kanan |

### Datasource methods (4841)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4843 | 4846 | `numberOfRowsInTableView:` | Jumlah baris tabel accessory status |
| 4848 | 4876 | `tableView:objectValueForTableColumn:row:` | Nilai sel untuk tabel status/checksum |
| 4878 | 4881 | `tableView:shouldEditTableColumn:row:` | Larang editing tabel status |

### Status accessory view (4884)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4886 | 4911 | `copyChecksumFromSheet:` | Copy baris status/checksum ke pasteboard |
| 4913 | 4916 | `setIsSavedInBundle:` | Set flag bundle-saved |

### Private API (4919)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4926 | 4952 | `_copyDatabase` | Validasi nama; dispatch copy DB ke thread |
| 4954 | 4985 | `_copyDatabaseWithDetails:` | Lakukan copy DB; select/refresh; lapor error |
| 4990 | 5036 | `_renameDatabase` | Lakukan rename DB; refresh/lapor hasil |
| 5043 | 5077 | `_addDatabase` | Buat database dengan charset/collation; select |
| 5082 | 5107 | `_alterDatabase` | Jalankan ALTER DATABASE; reset cache data DB |
| 5114 | 5147 | `_removeDatabase` | DROP DATABASE; refresh navigator/list; notify |
| 5157 | 5164 | `_selectDatabaseAndItem:` | Trampoline ke seleksi SADatabaseListManager |

### SADatabaseSelectionDelegate (5166)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5168 | 5171 | `currentSelectedDatabase` | Getter delegate DB terpilih |
| 5173 | 5180 | `setCurrentSelectedDatabase:` | Setter delegate DB terpilih |
| 5182 | 5185 | `currentSelectedTable` | Getter delegate tabel terpilih |
| 5187 | 5190 | `setCurrentSelectedTable:` | Setter delegate tabel terpilih |
| 5192 | 5195 | `historyStateIsModifying` | Lapor state history-controller |
| 5197 | 5200 | `setHistoryStateIsModifying:` | Set state history-controller |
| 5202 | 5205 | `isDatabaseConnected` | Lapor state koneksi live |
| 5207 | 5210 | `currentTableName` | Kembalikan nama tabel aktif |
| 5212 | 5215 | `chooseDatabaseButton` | Kembalikan popup button database |
| 5217 | 5220 | `selectMySQLDatabase:` | Minta koneksi memilih database |
| 5222 | 5225 | `updateHistoryEntries` | Minta history controller update entri |
| 5227 | 5230 | `rebuildDatabasesPopup` | Bangun ulang popup database |
| 5232 | 5235 | `endLoadingTask` | Akhiri task loading aktif |
| 5237 | 5240 | `resetDatabaseData` | Reset cache data database |
| 5242 | 5245 | `reattachTablesListConnection` | Reattach koneksi ke tables list |
| 5247 | 5250 | `refreshWindowTitle` | Refresh judul window |
| 5252 | 5257 | `presentUnableToSelectDatabaseAlertWithName:` | Tampilkan alert gagal pilih database |
| 5259 | 5262 | `selectTablesListItemWithNamed:` | Pilih item bernama di tables list |
| 5264 | 5267 | `setTableListSelectability:` | Set selectability tables list |
| 5269 | 5272 | `focusTableListFilter` | Fokus filter tables list |
| 5274 | 5277 | `focusTableList` | Fokus tables list |
| 5279 | 5282 | `processDatabaseChangedBundleTriggers` | Jalankan bundle trigger DB-berubah |
| 5284 | 5337 | `_processDatabaseChangedBundleTriggerActions` | Eksekusi command bundle trigger DB-berubah |
| 5342 | 5359 | `_addPreferenceObservers` | Daftarkan observer KVO NSUserDefaults (guarded) |
| 5364 | 5375 | `_removePreferenceObservers` | Hapus observer KVO NSUserDefaults (guarded) |

### SPDatabaseViewController / Getters (5377/5379)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5385 | 5388 | `databaseView` | Kembalikan view database master |
| 5393 | 5396 | `table` | Kembalikan tabel/view/nama terpilih |
| 5401 | 5404 | `tableType` | Kembalikan tipe tabel terpilih |
| 5409 | 5412 | `structureLoaded` | Kembalikan flag structure-loaded |
| 5417 | 5420 | `contentLoaded` | Kembalikan flag content-loaded |
| 5425 | 5428 | `statusLoaded` | Kembalikan flag status-loaded |

### Tab view control and delegate methods (5431)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5440 | 5456 | `switchToViewMode:` | Jalur switch view bersama; commit edit dulu |
| 5459 | 5461 | `viewStructure` | Switch ke view Structure |
| 5463 | 5465 | `viewContent` | Switch ke view Content |
| 5467 | 5474 | `viewQuery` | Switch ke Query editor; fokus text view |
| 5476 | 5486 | `viewStatus` | Switch ke Status; load info; fokus syntax view |
| 5488 | 5490 | `viewRelations` | Switch ke view Relations |
| 5492 | 5494 | `viewTriggers` | Switch ke view Triggers |
| 5496 | 5505 | `toggleRecordView:` | Toggle record view untuk host content/query |
| 5511 | 5525 | `setStructureRequiresReload:` | Tandai/reload structure sesuai view aktif |
| 5531 | 5541 | `setContentRequiresReload:` | Tandai/reload content sesuai view aktif |
| 5547 | 5557 | `setStatusRequiresReload:` | Tandai/reload info status sesuai view aktif |
| 5563 | 5573 | `setRelationsRequiresReload:` | Tandai/reload relations sesuai view aktif |
| 5579 | 5596 | `tabView:didSelectTabViewItem:` | Mulai load tab yang baru dipilih di thread |

### Table control (5599)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5608 | 5679 | `loadTable:ofType:` | Load/deselect tabel; mulai load tabel di thread |
| 5685 | 5735 | `_loadTabTask:` | Load data tab bila belum dimuat |
| 5740 | 5897 | `_loadTableTask` | Load tabel penuh di thread: cache, view, trigger |

### SPMySQLConnection delegate methods (5899)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5904 | 5914 | `willQueryString:connection:` | Log query ke console sesuai mode/pref |
| 5919 | 5924 | `queryGaveError:connection:` | Log error query ke console |
| 5929 | 5932 | `keychainPasswordForConnection:` | Minta password DB dari keychain |
| 5937 | 5942 | `credentialErrorMessageForConnection:` | Deskripsikan error token AWS IAM |
| 5949 | 5956 | `keychainPasswordForSSHConnection:` | Kembalikan password SSH keychain atau kosong |
| 5962 | 5965 | `noConnectionAvailable:` | Peringatkan tak ada koneksi |
| 5970 | 6006 | `connectionLost:` | Tampilkan sheet reconnect; kembalikan lost-decision |
| 6011 | 6018 | `showErrorWithTitle:message:` | Tampilkan alert warning non-fatal di main queue |
| 6023 | 6058 | `closeAndDisconnect` | Order out window, simpan history, teardown observer |
| 6060 | 6062 | `parentWindowControllerWindow` | Kembalikan parent window |

### SPPrintController (6064)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 6070 | 6090 | `printDocument` | Peringatkan konten besar; mulai operasi print |
| 6095 | 6112 | `startPrintDocumentOperation` | Mulai print di thread baru atau inline |
| 6117 | 6123 | `generateHTMLForPrintingCallback` | Cancel print thread saat cancel |
| 6129 | 6150 | `loadPrintWebViewWithHTMLString:` | Render HTML offscreen + jalankan print op |
| 6155 | 6284 | `generateHTMLForPrinting` | Bangun HTML print untuk view aktif |
| 6289 | 6321 | `generateTableInfoHTMLForPrinting` | Bangun HTML print untuk view table-information |
| 6328 | 6361 | `columnNames` | Nama header kolom untuk view yang di-print |
| 6366 | 6394 | `connectionInformation` | Bangun dictionary koneksi footer print |
| 6396 | 6421 | `documentWillClose:` | Saat tutup: disconnect, lepas observer, shutdown |
| 6425 | 6434 | `dealloc` | Safety-net cleanup observer; log dealloc |

## 4. Temuan menonjol / UNKNOWN

- **Dead code eksperimental — `compareDatabase:` (752–815).** Komentar menyebut "a basic experiment"; membandingkan tiap baris tabel **dengan dirinya sendiri** (`[row isEqualToArray:row]`, baris 801) dan hanya `NSLog`. **UNKNOWN** apakah ter-wire ke menu.
- **`showErrorSheetWith:` (953)** hanya di `.m`, dipanggil via `performSelector:` (5121); kontrak positional `[error objectAtIndex:0/1]` rapuh.
- **`clearConsole:` (1080)** tak ada di header; dipanggil dari `validateMenuItem:` (3096) dan toolbar.
- **Logika hasil terduplikasi.** `checkTable`/`analyzeTable`/`optimizeTable`/`repairTable`/`flushTable`/`checksumTable` (1606–1934) nyaris identik; hanya verb SQL + string pesan berbeda.
- **Dispatch bundle-trigger terduplikasi.** `_processDatabaseChangedBundleTriggerActions` (5284) dan loop inline di akhir `_loadTableTask` (5853–5895).
- **Literal peta encoding terduplikasi** di `encodingTagFromMySQLEncoding:` (1368) dan `mysqlEncodingFromEncodingTag:` (1402) — tabel charset 19 entri yang sama, dua arah.
- **`connectionID` (2233) punya hazard switch.** `case SPSSHTunnelConnection:` (2258–2264) tak ada `break` sebelum `case SPVaultConnection:` — kebetulan aman karena return eksplisit, tapi tak konsisten.
- **Restore sesi tidak konsisten.** `restoreSession` (4074) membaca `contentPageNumber` sebagai `@"pageNumber"` (4095) padahal `stateIncludingDetails:` menulisnya `@"contentPageNumber"` (3705) — **mismatch key**; restore nomor halaman tampak tak efektif.
- **`dispatch_sync` ke main dari main berpotensi deadlock** pada `_processDatabaseChangedBundleTriggerActions` (5288) dan `_loadTableTask`; caller tampak background-threaded. **UNKNOWN** apakah semua caller off-main.
