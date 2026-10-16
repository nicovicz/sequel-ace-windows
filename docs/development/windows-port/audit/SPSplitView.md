# Audit: `SPSplitView.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Views/SPSplitView.m`. Baseline commit `02c910932`.

- File: 1105 baris; header 71 baris.
- Peran: subclass `NSSplitView` yang menambah satu subview yang dapat di-collapse
  (animasi 30 fps berbasis timer + view pembungkus), constraint min/max per-index,
  view drag-handle tambahan, dan penerusan delegate ke delegate level-app.
- Total: 34 definisi method (SPSplitView 29 + HelperView 2 + RetainCycleBypass 2 + 1 initialize).

## 1. Peta struktur (anchor)

| Baris | Elemen |
| --- | --- |
| 31–33 | import (`SPSplitView.h`, `SPDateAdditions.h`, `<stdlib.h>`) |
| 35–48 | `@interface SPSplitView ()` — deklarasi method privat |
| 50–58 | `@interface SPSplitViewHelperView : NSView` |
| 60–68 | `@interface SPSplitViewAnimationRetainCycleBypass : NSObject` |
| 70–987 | `@implementation SPSplitView` |
| 992–1076 | `@implementation SPSplitViewHelperView` |
| 1081–1104 | `@implementation SPSplitViewAnimationRetainCycleBypass` |
| 473–478 | Komentar mendokumentasikan method yang **dihapus** (bukan kode) |

## 2. Inventaris ivar

**Tak ada `@property`/`@synthesize`** di kedua file; semua state ivar mentah.

### `SPSplitView` (`.h:37–55`)
| `.h` | Ivar | Catatan |
| --- | --- | --- |
| 38 | `__weak id<NSSplitViewDelegate, AllowSplitViewResizing> delegate` | Menutupi delegate `NSSplitView` sendiri; di-assign langsung m:115; seed dari `[super delegate]` m:690 |
| 40 | `IBOutlet NSButton *collapseToggleButton` | IB, di-toggle m:98, 142, 154, 203 |
| 41 | `IBOutlet NSView *additionalDragHandleView` | IB, m:606-613 |
| 43 | `NSUInteger collapsibleSubviewIndex` | `NSNotFound` = unset (m:674) |
| 44 | `BOOL collapsibleSubviewCollapsed` | flag otoritatif (m:202) |
| 46 | `uint64_t animationStartTime` | timestamp ns monotonic (m:231) |
| 47 | `double animationDuration` | detik; default 0.25, ×10 dengan Shift (m:234-247) |
| 48 | `NSTimer *animationTimer` | sekaligus flag "sedang animasi" (m:181, 502, 744) |
| 49 | `SPSplitViewAnimationRetainCycleBypass *animationRetainCycleBypassObject` | pemegang target timer (m:250) |
| 50–51 | `float animationStartSize/animationTargetSize` | (m:206/217) |
| 53–54 | `NSMutableArray *viewMinimumSizes/viewMaximumSizes` | per-index; max default `FLT_MAX` (m:681-687/707) |

Class extension (m:37–47): deklarasi 7 method, semua terimplementasi.

### `SPSplitViewHelperView`
`NSView *wrappedView` (m:52) strong; hanya dilepas via `restoreOriginalView` (m:1062-1063); tak ada `dealloc`.

### `SPSplitViewAnimationRetainCycleBypass`
`__unsafe_unretained SPSplitView *parentSplitView` (m:62) — komentar m:1088 menyebut "weak link" padahal unretained.

## 3. Semua `#pragma mark`

`75/76 (Setup and teardown), 110/111 (Delegate management), 118/119 (Collapsible subview management), 258/259 (Additional drag handle view), 275/276 (Constraint management), 312/313 (Sizing), 452/453 (Delegate method overrides), 648/649 (Delegate method forwarding), 666/667 (Private API), 711/713 (tanpa judul), 927 (tanpa judul), 973 (tanpa judul), 989/990 (Animation transition view class), 1078/1079 (Retain cycle avoidance class)`.

## 4. Tabel method (urut baris)

### `@implementation SPSplitView` (70–987)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 72 | 73 | `+initialize` | Override kosong |
| 78 | 84 | `initWithFrame:` | `super` lalu `_initCustomProperties` |
| 86 | 92 | `initWithCoder:` | Jalur nib; `super` lalu `_initCustomProperties` |
| 94 | 99 | `awakeFromNib` | Sinkron tombol collapse ke flag |
| 101 | 108 | `dealloc` | Invalidate timer animasi, hapus observer notifikasi |
| 113 | 116 | `setDelegate:` | Simpan delegate app di ivar sendiri; `super` tak dipanggil |
| 125 | 144 | `setCollapsibleSubviewIndex:` | Set/clear index collapsible, expand yang lama dulu |
| 151 | 155 | `setToggleCollapseButton:` | Simpan tombol IB + sinkron state |
| 160 | 167 | `isCollapsibleSubviewCollapsed` | Kembalikan flag; NO bila index unset |
| 174 | 182 | `isSubviewCollapsed:` | Override; YES hanya bila benar-benar collapse, tak animasi |
| 187 | 190 | `toggleCollapse:` | Flip state collapse dengan animasi |
| 196 | 256 | `setCollapsibleSubviewCollapsed:animate:` | Master collapse/expand: bungkus view, konfigurasi/start timer |
| 266 | 273 | `setAdditionalDragHandleView:` | Validasi window sama, simpan drag handle tambahan |
| 283 | 293 | `setMinSize:ofSubviewAtIndex:` | Simpan minimum per-index, validasi terhadap max |
| 300 | 310 | `setMaxSize:ofSubviewAtIndex:` | Simpan maximum per-index, validasi terhadap min |
| 331 | 450 | `adjustSubviews` | Layout inti: solve ukuran, backing-align, terapkan frame |
| 460 | 471 | `splitView:canCollapseSubview:` | Teruskan izin collapse, else hanya view collapsible |
| 485 | 507 | `splitView:shouldHideDividerAtIndex:` | Sembunyikan divider tunggal dekat subview collapse |
| 518 | 549 | `splitView:constrainMinCoordinate:ofSubviewAt:` | Turunkan minimum divider dari min/max berdekatan |
| 560 | 591 | `splitView:constrainMaxCoordinate:ofSubviewAt:` | Turunkan maximum divider dari min/max berdekatan |
| 599 | 616 | `splitView:additionalEffectiveRectOfDividerAtIndex:` | Konversi frame drag handle ke rect drag divider |
| 624 | 646 | `splitViewDidResizeSubviews:` | Lacak collapse/expand akibat drag, teruskan notifikasi |
| 651 | 653 | `splitView:constrainSplitPosition:ofSubviewAt:` | No-op; kembalikan posisi apa adanya |
| 655 | 660 | `splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:` | Teruskan rect divider efektif, else rect diusulkan |
| 662 | 664 | `splitView:shouldAdjustSizeOfSubview:` | Selalu YES |
| 669 | 693 | `_initCustomProperties` | Zero ivar, alokasi array ukuran, wire delegate ke self |
| 699 | 709 | `_ensureDefaultSubviewSizesToIndex:` | Pad array min/max dengan 0 dan `FLT_MAX` |
| 720 | 925 | `_suggestedSizesForTargetSize:respectingSpringsAndStruts:respectingConstraints:` | Solver constraint/spring tiga mode; nil bila infeasible |
| 932 | 938 | `_startPositionOfView:` | Origin lengthwise view |
| 943 | 949 | `_lengthOfView:` | Panjang lengthwise view |
| 954 | 960 | `_setStartPosition:ofView:` | Set origin lengthwise |
| 965 | 971 | `_setLength:ofView:` | Set ukuran lengthwise |
| 979 | 985 | `_isViewResizable:` | Tes autoresizing mask pada sumbu split |

### `@implementation SPSplitViewHelperView` (992–1076)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 999 | 1034 | `initReplacingView:inVerticalSplitView:` | Bungkus view untuk animasi |
| 1040 | 1074 | `restoreOriginalView` | Tukar kembali view, restore responder + redraw |

### `@implementation SPSplitViewAnimationRetainCycleBypass` (1081–1104)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1083 | 1092 | `initWithParent:` | Simpan parent tak-retained sebagai target timer |
| 1094 | 1102 | `_animationStep:` | Tick timer: `adjustSubviews` lalu `setNeedsLayout` |

## 5. Temuan menonjol / UNKNOWN

**Layout engine (`adjustSubviews`, 331–450)**
- Tiga-pass fallback: springs+constraint → constraint saja → tanpa constraint (347, 351, 356), masing-masing return nil saat gagal (831–836). Bail ke AppKit bila solver hasilkan terlalu sedikit ukuran (360–363).
- Buffer C `viewFramesAdjusted = calloc(sizeof(NSRect), viewCount)` (369) **tanpa NULL-check**; di-free di 446 — semua jalur return setelah 369 mencapai `free` (tak ada early exit 373–445).
- Sisa pembulatan didistribusi ke view non-nol terakhir + geser successor (410–437). TODO 412 soal `NSAlignAllEdgesNearest` vs `Outwards`/`Inwards`.

**Animasi collapse (196–256)**
- Wrapping hanya saat collapse (212–215). Target size expand diukur dari subview pertama helper (219). Durasi: 0.25 s (234), ×10 saat Shift ditahan (237–239), lalu diskala fraksi view yang berubah (241–247); bisa hasilkan `animationDuration == 0` (mis. `animationStartSize == 0` di 244).
- Timing pakai jam monotonic `clock_gettime_nsec_np(CLOCK_MONOTONIC)` (`SPDateAdditions.m:44-53`). Timer 30 fps `NSRunLoopCommonModes`, target = bypass object, `repeats:YES` (253–254).
- Penyelesaian dideteksi dalam solver, bukan timer: progress clamp 1 (781), invalidate timer (785–787), unwrap helper (790–792).
- `isSubviewCollapsed:` sengaja NO saat `animationTimer` hidup (181); `shouldHideDividerAtIndex:` juga hanya menyembunyikan setelah animasi berhenti (501–506).

**Retain-cycle bypass (60–68, 1081–1102)**
- `NSTimer` menahan target; bypass memutus siklus dengan menahan parent `__unsafe_unretained` (62). Komentar 1088 bilang "weak" — **mismatch komentar vs kode**; dangling bila split view mati saat timer masih terjadwal. Mitigasi di `dealloc` hanya invalidate timer (103–105).
- `_animationStep:` panggil `adjustSubviews` lalu `setNeedsLayout:YES` (1096, 1101); komentar 1098–1100 menjelaskan workaround layer-backing 10.14 untuk divider basi.

**TODO leak (semua tampak STALE — UNKNOWN)**
| Baris | Komentar | Penilaian |
| --- | --- | --- |
| 250 | `// TODO: leaks` pada bypass object | Dimiliki ivar strong h:49; target timer dilepas saat `invalidate` (104, 229, 786) — di ARC tidak leak |
| 729 | `// TODO: jcs - all these callocs leak` | `originalSizes`/`minSizes`/`maxSizes` di-free di jalur gagal (832–834) & sukses (918–920); `sizesCalculated`/`resizeProportions` di 921–922. Tak ada leak ditemukan |
| 735 | `// TODO: this leaks` pada `outputSizes` | `NSMutableArray` autoreleased (924); dimiliki caller (347–357) — bukan leak |

**Smell korektness (disitir, tak diperbaiki)**
- **Potensi heap overwrite:** di 852–855, saat delegate veto resize, `resizeProportions[1]` ditulis tanpa syarat; dengan satu subview, array dialokasikan `subviewCount` float (840) → out-of-bounds.
- **Off-by-one:** m:131 menolak `subviewIndex > count` tapi menerima `== count`, yang lalu meng-index `objectAtIndex:` di 205 dan 466.
- **Typo exception:** `"Specified a collpasible subview index which doesn't exist"` (132).
- **Dispatch tak sinkron:** `_isViewResizable:` (979–985) melaporkan struts, tapi `splitView:shouldAdjustSizeOfSubview:` selalu YES (662–664).
- **Delegate shadowing:** `setDelegate:` (113–116) tak panggil `super`, sementara `_initCustomProperties` set `[super setDelegate:self]` (692) + tangkap nilai lama (690).
- **`+initialize` kosong** (72–73).
- **NaN edge (UNKNOWN):** bila `animationDuration == 0` dan `_animationStep:` pertama pada elapsed 0, m:780 bagi 0/0 → NaN; clamp `> 1` (781) tak menangkap NaN.
- `_suggestedSizes…` pad ke `subviewCount + 1` (737) sedangkan `_initCustomProperties` pad ke `l-1` (688) — array bisa satu lebih panjang.
- `SPSplitViewHelperView.wrappedView` bukan `__weak` + tak ada `dealloc` (52); hanya dilepas via `restoreOriginalView` (1062–1063).
