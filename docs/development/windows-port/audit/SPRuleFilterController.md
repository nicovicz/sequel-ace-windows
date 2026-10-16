# Audit: `SPRuleFilterController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPRuleFilterController.m`. Baseline commit
`02c910932`.

- File: 2481 baris; header `SPRuleFilterController.h` 278 baris.
- Peran: controller editor filter berbasis rule untuk content tabel; memiliki
  `NSRuleEditor`, serialisasi/restore tree kriteria ke/dari dictionary, dan
  merender ke klausa SQL WHERE.
- Total: 52 method publik/privat + 5 kelas helper (RuleNode, ColumnNode, StringNode,
  OpNode, ArgNode, ConnectorNode, EnableNode, ModelContainer) + 2 fungsi C.

## 1. Konstanta & kelas helper

### Konstanta atas file (`.m:42-125`)
| Baris | Item |
| --- | --- |
| 42–49 | enum `RuleNodeType` (`Column, String, Operator, Argument, Connector, Enable`) |
| 51 | `SPRuleFilterHeightChangedNotification` |
| 56–124 | key filter terserialisasi: `SerFilterClass`(56), `SerFilterClassGroup`(60), `SerFilterClassExpression`(64), `SerFilterGroupIsConjunction`(70), `SerFilterGroupChildren`(75), `SerFilterExprColumn`(83), `SerFilterExprType`(88), `SerFilterExprComparison`(96), `SerFilterExprValues`(105), `SerFilterExprDefinition`(113, tidak dipersist), `SerFilterExprEnabled`(118), `SerFilterExprPendingStarter`(124) |

### Ivar `.h` (41–72)
IBOutlet `filterRuleEditor`, `tableDataInstance`, `tableDocumentInstance`(weak),
`tablesListInstance`, `tableContentViewBelow`, `filterButton`, `addFilterButton`,
`rootConjunctionPopUp` (42–49); `columns`, `contentFilters`, `numberOfDefaultFilters`
(51–53); `_modelContainer` (55); `contentFilterManager` (57); `preferredHeight` (59);
`target`(61)/`action`(62); `enabled`/`rootIsConjunction` (64–65);
`opNodeCacheVersion`, `isDoingChangeCausedOutsideOfRuleEditor`, `previousRowCount` (67–69);
`dropBox` (71).

### Kelas helper privat (`.m`)
`RuleNode` (128–137, ivar `type`), `ColumnNode` (139–149, `name/typegrouping/operatorCache/opCacheVersion`),
`StringNode` (151–155, `value`), `OpNode` (157–171, `parentColumn/settings/filter`),
`ArgNode` (173–181, `filter/argIndex/initialValue`), `ConnectorNode` (183–189, `filter/labelIndex`),
`EnableNode` (191–199, `initialState/allowsMixedState` + `pendingStarter`), `ModelContainer` (230–236, `model`).
Class extension `()` (240–286): `preferredHeight` readwrite (242), `pendingStarter` strong (244), prototipe method (247–285).

## 2. Semua `#pragma mark` (urut baris)

`126, 201, 238, 2231, 2444` — **kelimanya tanpa judul**. Tidak ada `#pragma mark`
bernama, dan **tidak ada pragma di dalam `@implementation SPRuleFilterController`
(288–2229)**. Komentar `// KVO` biasa di 2459.

## 3. Tabel method (urut baris)

### `@implementation SPRuleFilterController` (288–2229)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 294 | 350 | `init` | Init state; muat default ContentFilters.plist; hitung default filter |
| 352 | 386 | `awakeFromNib` | Bind `rows` ke model; lokalisasi popup; pasang drop box |
| 388 | 391 | `dropBoxView` | Kembalikan drop box view |
| 393 | 399 | `dropBoxReservedHeight` | Ruang tetap 40pt di bawah rule editor |
| 401 | 406 | `focusFirstInputField` | Fokus field teks pertama di baris root |
| 408 | 425 | `_focusOnFieldInSubtree:` | Fokus rekursif NSTextField pertama di subtree baris |
| 427 | 463 | `setColumns:` | Bangun ulang `ColumnNode` dari def; reset konjungsi root |
| 465 | 518 | `ruleEditor:numberOfChildrenForCriterion:withRowType:` | Delegate: jumlah child per node/tipe baris |
| 520 | 592 | `ruleEditor:child:forCriterion:withRowType:` | Delegate: bangun child kolom/op/arg/connector |
| 594 | 725 | `ruleEditor:displayValueForCriterion:inRow:` | Delegate: bangun checkbox/menu item/text field |
| 727 | 758 | `control:textView:doCommandBySelector:` | Shift-delete hapus rule atau reset filter |
| 760 | 785 | `_checkboxClicked:` | Sinkron checkbox enable down/up; refresh preview |
| 796 | 814 | `_updateCheckedStateDownwardsFromCompoundRow:to:` | Propagasi state checkbox ke child row |
| 825 | 833 | `_updateCheckedStateForRow:to:` | Set state checkbox enable satu baris |
| 847 | 877 | `_updateCheckedStateUpwardsFromCompoundRow:` | Agregasi state child ke checkbox parent |
| 893 | 922 | `_recalculateCheckboxStatesFromRow:` | Hitung ulang state compound bottom-up |
| 936 | 947 | `_checkboxStateForRow:` | State checkbox enable untuk satu baris |
| 949 | 956 | `_textFieldAction:` | Return/enter di field argumen memicu filtering |
| 962 | 967 | `controlTextDidChange:` | Preview live; enable pending starter saat mengetik |
| 975 | 987 | `_updateFilterPreview` | Tampilkan klausa WHERE aktif di drop box |
| 989 | 1079 | `_menuItemInRuleEditorClicked:` | Tangani pilih kolom/operator; bangun ulang kriteria baris |
| 1092 | 1137 | `_pretendPlayRuleEditorForCriteria:displayValues:inRow:tryingToPreserveOldCriteria:displayValues:` | Rekonstruksi kriteria/displayValues setelah elemen berubah |
| 1139 | 1142 | `filterTable:` | Panggil target/action filter |
| 1148 | 1155 | `_invokeTarget:action:withObject:` | Jalankan target/action runtime, suppress warning ARC |
| 1160 | 1163 | `_invokeFilterTargetActionWithObject:` | Panggil target/action filter controller |
| 1165 | 1177 | `resetFilter` | Bersihkan model, reset konjungsi, toggle tombol filter |
| 1179 | 1182 | `addFilter:` | Tombol: tambah ekspresi filter |
| 1184 | 1194 | `_resize` | Post notifikasi tinggi berubah bila tinggi berubah |
| 1201 | 1241 | `ruleEditorRowsDidChange:` | Sinkron tinggi/tombol/checkbox; tangani penghapusan baris terakhir |
| 1243 | 1254 | `_updateButtonStates` | Show/hide & enable kontrol filter/add/konjungsi |
| 1259 | 1262 | `rootIsConjunction` | Getter pilihan AND/OR root |
| 1267 | 1271 | `setRootIsConjunction:` | Set konjungsi root; mirror ke popup |
| 1277 | 1280 | `_isConjunctionChoiceNode:` | Uji string node adalah pilihan AND/OR vs explainer |
| 1286 | 1291 | `+_conjunctionExplainerNode` | Bangun node label explainer statis untuk group |
| 1296 | 1300 | `_syncRootConjunctionPopUp` | Pilih item popup sesuai `rootIsConjunction` |
| 1305 | 1311 | `_rootConjunctionPopUpChanged:` | Simpan pilihan popup AND/OR; refresh preview |
| 1313 | 1319 | `dealloc` | Unbind rows; hapus observer notifikasi |
| 1324 | 1455 | `_compareTypesForColumn:` | Bangun list `OpNode` operator (termasuk separator/Edit Filters) |
| 1457 | 1464 | `_editFiltersAction:` | Buka content filter manager dari item menu |
| 1466 | 1473 | `openContentFilterManagerForFilterType:` | Tampilkan sheet content filter manager |
| 1475 | 1483 | `_contentFiltersHaveBeenUpdated:` | Invalidate cache op; reload kriteria |
| 1485 | 1492 | `_ensureValidOperatorCache:` | Bangun ulang cache operator kolom bila stale |
| 1494 | 1497 | `isEmpty` | Apakah model container tanpa baris |
| 1499 | 1511 | `addFilterExpression` | Sisip baris atas sederhana baru; fokus |
| 1516 | 1527 | `addStarterFilterExpression` | Tambah baris starter unchecked menunggu edit |
| 1532 | 1536 | `pendingStarter` | Buat tracker pending-starter (lazy) |
| 1542 | 1549 | `_enablePendingStarterInRow:` | Centang baris starter saat edit pertama; refresh UI |
| 1555 | 1605 | `_makeSerializedRuleForColumn:value:isNull:` | Bangun rule terserialisasi dengan operator default/IS NULL |
| 1611 | 1637 | `appendFilterForColumn:value:isNull:` | Append rule cell yang di-drop sebagai baris top-level |
| 1643 | 1676 | `replaceFilterAtRow:forColumn:value:isNull:` | Ganti rule top-level yang dialamatkan dengan cell yang di-drop |
| 1681 | 1711 | `addEmptyFilterGroup` | Append group bersarang konjungsi berlawanan + starter |
| 1717 | 1759 | `addEmptyFilterRow` | Centang baris kosong atau append baris kosong di akhir |
| 1761 | 1764 | `view` | Kembalikan rule editor view |
| 1766 | 1769 | `isEnabled` | Getter flag enabled |
| 1771 | 1775 | `setEnabled:` | Enable hanya bila ada kolom; update tombol |
| 1777 | 1802 | `sqlWhereExpressionWithBinary:error:` | Flatten filter terserialisasi; render string SQL WHERE |
| 1804 | 1807 | `serializedFilter` | Serialisasi publik tanpa definisi filter |
| 1809 | 1818 | `_serializedFilterIncludingFilterDefinition:withDisabled:` | Serialisasi item root ke dict konjungsi root |
| 1820 | 1886 | `_serializeSubtree:includingDefinition:withDisabled:` | Serialisasi rekursif group atau baris ekspresi |
| 1893 | 1917 | `restoreSerializedFilters:` | Restore baris/konjungsi dari dict terserialisasi |
| 1919 | 2047 | `_restoreSerializedFilter:` | Bangun ulang kriteria+displayValues baris dari node |
| 2049 | 2058 | `+makeSerializedFilterForColumn:operator:values:` | Bangun dict ekspresi tunggal |
| 2060 | 2068 | `_columnForName:` | Cari `ColumnNode` terkonfigurasi by nama |
| 2070 | 2081 | `_operatorNamed:forColumn:` | Cari `OpNode` operator by label menu |
| 2102 | 2136 | `+_flattenSerializedFilter:` | Flatten group bersarang/beranak-tunggal yang redundan |
| 2138 | 2216 | `+_writeFilterTree:toString:wrapInParenthesis:binary:error:` | Render rekursif tree terserialisasi ke SQL |
| 2218 | 2227 | `_doChangeToRuleEditorData:` | Tandai perubahan model sebagai eksternal dalam blok |

**Fungsi C**: `void _addIfNotNil(NSMutableArray*,id)` (1888–1891),
`BOOL SerIsGroup(NSDictionary*)` (2083–2086).

### `@implementation RuleNode` (2233–2253)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2237 | 2239 | `hash` | Hash by tipe node |
| 2241 | 2246 | `isEqual:` | Kesetaraan by kelas + tipe |
| 2248 | 2251 | `isViableReplacementFor:` | Cek tipe sama untuk replacement elemen baris |

### `@implementation ColumnNode` (2255–2290)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2262 | 2269 | `init` | Set type=Column; cache version 0 |
| 2271 | 2274 | `description` | String debug dengan nama + pointer |
| 2276 | 2278 | `hash` | Hash dari name, typegrouping, super |
| 2280 | 2288 | `isEqual:` | Sama bila name dan typegrouping cocok |

### `@implementation StringNode` (2292–2315)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2296 | 2302 | `init` | Set type=String |
| 2304 | 2306 | `hash` | Hash dari value + super |
| 2308 | 2313 | `isEqual:` | Sama bila string value cocok |

### `@implementation OpNode` (2317–2352)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2323 | 2329 | `init` | Set type=Operator |
| 2332 | 2334 | `hash` | Hash dari parentColumn, settings, super |
| 2336 | 2341 | `isEqual:` | Sama bila settings + parentColumn cocok |
| 2343 | 2345 | `isViableReplacementFor:` | Tipe sama + kolom parent sama |
| 2347 | 2350 | `name` | Label menu dari dict filter (`MenuLabel`) |

### `@implementation ArgNode` (2354–2386)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2360 | 2366 | `init` | Set type=Argument |
| 2369 | 2372 | `hash` | Hash dari argIndex, filter (abaikan initialValue) |
| 2374 | 2380 | `isEqual:` | Sama bila filter + argIndex cocok |
| 2382 | 2384 | `isViableReplacementFor:` | Tipe sama + arg index sama |

### `@implementation ConnectorNode` (2388–2413)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2393 | 2399 | `init` | Set type=Connector |
| 2402 | 2404 | `hash` | Hash dari labelIndex, filter, super |
| 2406 | 2411 | `isEqual:` | Sama bila filter + labelIndex cocok |

### `@implementation EnableNode` (2415–2442)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2421 | 2429 | `init` | Set type=Enable; initial on, mixed off |
| 2431 | 2433 | `hash` | Hash gabungan super + flag initial/mixed |
| 2435 | 2440 | `isEqual:` | Sama bila flag initial + mixed cocok |

### `@implementation ModelContainer` (2446–2481)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2450 | 2456 | `init` | Buat array model backing |
| 2461 | 2464 | `insertObject:inModelAtIndex:` | KVO insert ke model |
| 2466 | 2469 | `removeObjectFromModelAtIndex:` | KVO remove dari model |
| 2471 | 2474 | `insertModel:atIndexes:` | KVO insert banyak |
| 2476 | 2479 | `removeModelAtIndexes:` | KVO remove banyak |

## 4. Temuan menonjol / UNKNOWN

**Tree filter terserialisasi**
- Key di `.m:56-124`. **Build** `_serializedFilterIncludingFilterDefinition:withDisabled:` (1809–1818) loop item root → `_serializeSubtree:…` (1820–1886); group di 1837–1841, ekspresi di 1865–1872; definisi filter hanya bila `includeDefinition` (1880–1883). Entry publik `serializedFilter` (1804–1807, `includeDefinition:NO withDisabled:YES`).
- **Restore** `restoreSerializedFilters:` (1893–1917) → `_restoreSerializedFilter:` (1919–2047); group 1923–1949, ekspresi 1950–2041, gagal via `goto fail` (2045–2046) saat kolom/operator/tipe/argumen tak cocok.
- **Rendering SQL** `sqlWhereExpressionWithBinary:error:` (1777–1802) → `_flattenSerializedFilter:` (2102–2136) → `_writeFilterTree:toString:wrapInParenthesis:binary:error:` (2138–2216); SQL dibangun `SPTableFilterParser` (2195–2212); error pakai `SPErrorDomain` 0-3 (2173–2211).
- **UNKNOWN**: `SerFilterExprDefinition` didokumentasikan "not designed to be serialized to disk" (111–113) tapi tetap key di tree; pengecualiannya bergantung penuh pada flag `includeDefinition` (dicek 1880, 1784).

**Konjungsi root**
- Ivar `rootIsConjunction` (`.h:65`); reset YES di `setColumns:` (459) dan `resetFilter` (1175); dibaca serialisasi (1817) dan merge helper (1631–1633, 1652–1655, 1701–1703). Getter/setter (1259–1271) mirror ke popup via `_syncRootConjunctionPopUp` (1296–1300).

**Hook merge cell-filter**
- `_makeSerializedRuleForColumn:value:isNull:` (1555–1605) — pilih operator `IS NULL` via pencarian klausa (1579–1586) atau operator 1-arg pertama (1590–1596); **return nil untuk menolak** drop tak didukung.
- `appendFilterForColumn:value:isNull:` (1611–1637) merge via `SARuleFilterRootConjunction appendingRule:to:rootIsConjunction:` (1631–1633); `replaceFilterAtRow:…` (1643–1676) via `replacingRule:atRow:in:rootIsConjunction:` (1652–1655), menolak target group; komentar panjang 1658–1671 soal selisih row-index vs editor-row.
- `addEmptyFilterGroup` (1681–1711) membangun starter menunggu (`SerFilterExprEnabled=NO`, `SerFilterExprPendingStarter=YES`, 1693–1694) → `treeAddingGroupWithStarter:to:rootIsConjunction:` (1701–1703).
- **UNKNOWN**: `SARuleFilterRootConjunction` juga mengekspos `isUntouchedStarterTree(_:)` dan `extendingMarkedRoot(_:withRule:)` — **tak dipanggil dari `.m` ini**; call site tak diketahui/kemungkinan dead.

**Kolaborator Swift eksternal**: `SARuleFilterRootConjunction` (1631, 1652, 1701, 1817, 1898),
`SARuleFilterRootRestorePlan` (1898), `SARuleFilterPendingStarter` (244, 631, 763, 806, 1521–1732),
`SARuleFilterConjunctionRowPresentation` (1279, 1289), `SARuleFilterResizePolicy` (1211, 1224),
`SPRuleFilterDropBox` (378–390, 985), `SPTableFilterParser` (2195), `SACellFilterOperator` (344, 1347–1348).

**Lain**
- Pelacakan **pending-starter** lintas-cutting: ditandai saat dibuat (1516–1527), dipertahankan lewat serialisasi (1875–1879), restore (1972–1979), dibersihkan saat edit/klik (763, 806, 965, 1019, 1542–1549, 1622, 1672, 1730–1732).
- Propagasi **checkbox tri-state** kuartet mandiri (796–947).
- **`init` efek samping**: membaca `ContentFilters.plist` dan bisa `NSBeep()` saat gagal (311–347).
- **Thread-safety**: banyak method ber-assert main-thread-only (`.h:77,84,93,108,116,128,148,155,162,173,196,216,226,236,246,268`); `makeSerializedFilterForColumn:operator:values:` didokumentasikan thread-safe (`.h:141`).
