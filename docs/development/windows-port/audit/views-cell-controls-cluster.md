# Audit: Table/cell/control view cluster

Audit method-by-method klaster view. `Start`/`End` = nomor baris di file masing-masing.
Baseline commit `02c910932`.

File: `TableViews/SPTableView.m` (340), `SPWindow.m` (201), `Cells/ImageAndTextCell.m` (205),
`Cells/SPActivityTextFieldCell.m` (353), `Cells/SPTextAndLinkCell.m` (283),
`Controls/SPColorSelectorView.m` (459), `Controls/SPComboPopupButton.m` (304).

**Catatan anchor**: method ditulis tanpa spasi setelah tanda (`- copyWithZone:`) **tidak** match `^[-+] (`. Satu kasus: `ImageAndTextCell.m:53`.

---

## 1. `SPTableView.m` (340)

Subclass `NSTableView`: seleksi baris right-click, penekanan context-menu saat DB sibuk, handling key Enter/Escape/Tab, hook double-click area kosong.

### Ivar/konstanta
`.h:33` `tabEditingDisabled` (BOOL), `.h:34` `emptyDoubleClickAction` (SEL), `.h:37` property, synthesize 58.
`@protocol SPTableViewDelegate` (38–42: `cancelRowEditing` 40); kategori `NSTableView (ApplePrivate)` (44–46: `_allowTabbingIntoCells` 45); extension (48–54). `NSTableViewStylePlain` (65); `SPFavorites` (132).

### `#pragma mark`
`76`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 60–74 | | `init` / `awakeFromNib` | Init + pasang double-action |
| 83 | 96 | `viewWillMoveToWindow:` | Registrasi observer sheet show/hide |
| 102 | 146 | `menuForEvent:` | Seleksi baris right-click, tekan menu saat sibuk |
| 148 | 151 | `acceptsFirstResponder` | Selalu YES |
| 196 | 199 | `_allowTabbingIntoCells` | Override method privat Apple, hormati tabbing disabled |
| 201 | 277 | `keyDown:` | Return/Enter edit, Escape cancel, Tab focus |
| 283 | 293 | `rightMouseDown:` | Lolos hanya bila delegate izinkan seleksi |
| 295–309 | | `setFont:` / `setEmptyDoubleClickAction:` | Font + selector |
| 315–338 | | `_doubleClickAction` / `_disableDoubleClickAction:` / `_enableDoubleClickAction:` | Double-click area kosong |

### Temuan
- **Key handling** (`keyDown:` 201–277): keyCode 36/76 Return/Enter (204), Escape 53 (243), `NSTabCharacter` (256); special-case `SPFieldMapperController` (206–225); whitelist-exclude `SPCustomQuery`/`SPQueryFavoriteManager`/`SPConnectionController` (227–229); delegasikan izin edit via `tableView:shouldEditTableColumn:row:` (232–233).
- **`@try/@catch` sekitar `super keyDown:`** (268–276) = scaffolding debug issue #2445, **re-raise**, konversi exception → `NSInternalInconsistencyException`.
- **Hit-testing context menu** (`menuForEvent:` 102–146) bandingkan **nama kelas** delegate (string-compare: `SPTablesList`, `SPQueryFavoriteManager`, `SPContentFilterManager`) alih-alih cek protokol — rapuh (126, 131, 136).
- ⚠️ **`setFont:` (295–304) tidak panggil `[super setFont:]`** — **UNKNOWN** intent; loop hanya menutup `dataCell` klasik, jadi tabel view-based tak dapat font.
- Label `pass_keyDown_to_super:` + `goto` (222, 267) — hack control-flow.

---

## 2. `SPWindow.m` (201)

Subclass `NSWindow`: shortcut keyboard alternatif (switch tab, Option+Escape context menu), undo manager per-tab, handling sheet-as-main.

### Ivar/konstanta
`.h:33` `isSheetWhichCanBecomeMain` (BOOL), `.h:36` property, synthesize 38.
Notifikasi `SPWindowSelectNextTabNotification` (104, 118), `SPWindowSelectPreviousTabNotification` (110, 124), `SPWindowToolbarDidToggleNotification` (175).

### `#pragma mark`
`40, 54 (Keyboard shortcut additions), 133 (Undo manager handling), 149 (Method overrides)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 42–52 | | `isResizable` / `isMiniaturizable` / `isReleasedWhenClosed` | Paksa true |
| 61 | 131 | `sendEvent:` | Intersep shortcut + context menu Option+Escape |
| 141 | 147 | `undoManager` | Rutekan undo ke dokumen tab terdepan |
| 155–180 | | `canBecomeMainWindow` / `toggleToolbarShown:` / `toggleFullScreen:` | Override |
| 182 | 199 | `validateMenuItem:` | Lokalkan judul menu Show/Hide Toolbar |

### Temuan
- **Intersep key** (`sendEvent:` 61–131): `}` (102), `{` (108), `NSRightArrowFunctionKey` (116), `NSLeftArrowFunctionKey` (122).
- **Option+Escape → right-click sintetis** (67–96): bangun `NSEvent` right-mouse-down (79–88) + `[NSMenu popUpContextMenu:...]` (90) terhadap `menuForEvent:` first responder. ⚠️ Guard cek `[theEvent type] == NSEventTypeKeyDown` + `charactersIgnoringModifiers` (62), dan **re-use event key asli** `theEvent` untuk `menuForEvent:` (90), bukan `anEvent` sintetis — **UNKNOWN** apakah disengaja.
- `isReleasedWhenClosed` (50–52) return YES tanpa syarat — mungkin mengejutkan untuk window ARC — **UNKNOWN**.
- `toggleFullScreen:` (178–180) pass-through murni ke super — **UNKNOWN** tujuannya.

---

## 3. `ImageAndTextCell.m` (205)

Subclass `NSTextFieldCell` (turunan Apple) yang menggambar gambar leading + dukung indentation level.

### Ivar/konstanta
`.h:41` `image` (NSImage*), `.h:42` `_indentationLevel` (NSInteger), `.h:37` `#define INDENT_AMOUNT 17`. Tanpa `@synthesize`/extension.

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 41–51 | | `init` / `dealloc` | Init (nil image) + teardown |
| 53 | 59 | `copyWithZone:` | Copy cell + deep-copy image (tanpa spasi → di luar anchor) |
| 61–72 | | `setImage:` / `image` | Akses image |
| 74–102 | | `imageFrameForCellFrame:` / `editWithFrame:inView:editor:delegate:event:` | Frame + edit offset |
| 104–117 | | `selectWithFrame:inView:editor:delegate:start:length:` | Offset seleksi |
| 119–183 | | `drawWithExpansionFrame:inView:` / `drawWithFrame:inView:` | Gambar image + teks |
| 185–203 | | `cellSize` / `setIndentationLevel:` / `IndentationLevel` | Ukuran + indent |

### Temuan
- **Drawing**: `drawWithFrame:` (154–183) + `drawWithExpansionFrame:` (119–152) split rect via `NSDivideRect` (159, 124, 133) + gambar `NSCompositingOperationSourceOver` (179, 145); beda offset vertikal (177 vs 143, yang terakhir `-1`).
- Tanpa hit-testing/drag di sini.
- ⚠️ `dealloc` (49–51) hanya `image = nil` — no-op di ARC.
- ⚠️ `setImage:` (61–67) simpan referensi **langsung (tanpa copy)**.
- `imageFrameForCellFrame:` ekspresi `(1 - MIN(1,INDENT_AMOUNT)) * 3` (81) = aritmetika dead saat `INDENT_AMOUNT==17`.

---

## 4. `SPActivityTextFieldCell.m` (353)

Subclass `ImageAndTextCell`: entri aktivitas dua-baris (nama + info) dengan tombol "link" cancel inline.

### Ivar/konstanta
`.h:36–45` ivar `activityName`, `activityInfo`, `contextInfo`, `cancelButton`, `drawState`, `mainStringColor`, `subStringColor`; property `.h:48–50`; synthesize 55–57. `#define FAVORITE_NAME_FONT_SIZE 12.0f` (34); `static inline SPTextLinkRectFromCellRect` (50–53); enum `SPLinkDrawState*` (`SPConstants.h:152–154`).

### `#pragma mark`
`319 (Private API)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 62–111 | | `init` / `encodeWithCoder:` / `copyWithZone:` | Init + coding |
| 116 | 197 | `drawInteriorWithFrame:inView:` | Gambar dua baris teks + tombol cancel |
| 199–210 | | `expansionFrameWithFrame:inView:` / `hitTestForEvent:inRect:ofView:` | Tekan tooltip + area hit |
| 216 | 283 | `trackMouse:inRect:ofView:untilMouseUp:` | Track klik cancel, kill proses |
| 285 | 299 | `cellSize` | Ukuran sel dua-baris |
| 304–317 | | `invertFontColors` / `restoreFontColors` | Warna teks saat seleksi |
| 324–351 | | `constructSubStringAttributedString` / `attributedStringForFavoriteName` / `mainStringAttributedStringAttributes` / `subStringAttributedStringAttributes` | Attributed string |

### Temuan
- **Drawing** (116–197): truncate main/sub dengan `"..."` di loop lebar (149–163); posisi `cancelButton` dari `SPTextLinkRectFromCellRect(cellFrame)` (168); state (171–175) toggle `SPLinkDrawStateHighlight`/`BackgroundHighlight`/`Normal`.
- **Hit-testing** selalu return `NSCellHitContentArea | NSCellHitTrackableArea` (209) — seluruh sel trackable (beda dari `SPTextAndLinkCell` yang region-test).
- ⚠️ **Track mouse + kill proses** (216–283): fast path delegasi super di luar `linkRect` (224); saat klik baca `contextInfo[@"type"]`/`[@"pid"]` + jalankan `/bin/sh` pipeline kill berbasis `egrep` (249–259) — `kill -9` sederhana dikomentari (254) + redirect keras `2&> /tmp/sp_kill_error.txt` (255, artefak debug). Penghapusan via `[[(id)controlView delegate] onMainThread] removeActivity:` (264). Pakai `SPBundleTaskScriptCommandFilePath` (import `SPTableInfo.h`, 32).
- ⚠️ `encodeWithCoder:` (89–92) hanya panggil super → field kustom **tidak** dipersist — **UNKNOWN** intent (padahal `copyWithZone:` menyalinnya).

---

## 5. `SPTextAndLinkCell.m` (283)

Subclass `NSTextFieldCell`: gambar "panah link" in-cell yang menjalankan target/action saat diklik.

### Ivar/konstanta
`.h:33–42` ivar `hasLink`, `linkActive`, `linkButton`, `linkTarget`, `linkAction`, `lastLinkColumn`, `lastLinkRow`, `drawState`; property `.h:45`; synthesize 35. `static inline SPTextLinkRectFromCellRect` (40–43); enum `SPLinkDrawState*`.

### `#pragma mark`
`45/46 (Setup and teardown), 87/88 (Enabling link functionality), 113/114 (Drawing and interaction), 256/257 (Information getters)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 51–85 | | `initWithCoder:` / `encodeWithCoder:` / `copyWithZone:` | Init + coding |
| 94 | 111 | `setTarget:action:` | Simpan target/action + buat link button |
| 120 | 164 | `drawInteriorWithFrame:inView:` | Gambar teks, sisakan ruang, gambar panah |
| 169 | 188 | `hitTestForEvent:inRect:ofView:` | Return trackable hanya di atas link rect |
| 194 | 254 | `trackMouse:inRect:ofView:untilMouseUp:` | Track klik link, catat row/kolom, fire action |
| 262–281 | | `getClickedColumn` / `getClickedRow` / `expansionFrameWithFrame:inView:` | Getter + tekan tooltip |

### Temuan
- **Drawing** (120–164): fast-path tanpa/tak aktif link delegasi super dengan rect bergeser (124–128); else sisakan `width-18` untuk panah (131) + gambar `linkButton` via `drawInteriorWithFrame:` (163); image panah tukar saat state berubah (145–161).
- **Hit-testing** (169–188): region-test `p.x` terhadap `linkRect` (179) → `NSCellHitTrackableArea`; else `NSCellHitEditableTextArea` (186) — kontras dengan `SPActivityTextFieldCell`.
- **Track mouse** (194–254): saat klik catat `lastLinkColumn`/`lastLinkRow` via `columnAtPoint:`/`rowAtPoint:` (226–227) + perform selector dengan suppression `#pragma clang diagnostic` sekitar `-Warc-performSelector-leaks` (232–235, call 234).
- ⚠️ `encodeWithCoder:` (70–73) tak encode link state — **UNKNOWN** intent.
- Tanpa `dealloc`.

---

## 6. `SPColorSelectorView.m` (459)

`NSView` yang merender hingga 7 titik warna (+ titik "X" clear) dengan highlight hover, notifikasi balik KVO/Cocoa-binding, dan callback delegate saat seleksi.

### Ivar/konstanta
`.h:47–59` ivar `trackingAreas`, `selectedTag`, `hoverTag`, `trackEntered`, `observer` (weak), `observerKeyPath`, `observingSelfSelectedTag`, `delegate`, `colorList`; property `.h:62,67`; synthesize 55–56. `kTrackerKey` (59); `enum trackingAreaIDs` (63–73: None=-1, 0..6); protocol `.h:71–78`; extension (45–51).

### `#pragma mark`
`236/237 (Custom Drawing), 356/357 (Mouse Handling)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 80–147 | | `initWithFrame:` / `bind:toObject:withKeyPath:options:` / `unbind:` / `observeValueForKeyPath:...` | Setup + binding |
| 152–234 | | `rectForColorViewAtIndex:` / `gradientForTag:` / `setupTrackingAreas` | Geometri + tracking |
| 244–354 | | `drawRect:` / `_drawDotBevelStyleWithGradient:insideRect:` / `_drawDotFlatStyleWithColor:insideRect:` | Gambar |
| 365–436 | | `getTrackerIDFromDict:` / `mouseEntered:` / `mouseExited:` / `rightMouseUp:` / `mouseUp:` | Mouse |
| 444–457 | | `dealloc` | Hapus KVO sebelum dealloc |

### Temuan
- **Drawing** (244–303): ring seleksi hanya saat `index > kTrackingAreaNone && index == selectedTag` (260–270); ring hover (271–281) saat `index == hoverTag && trackEntered`; glyph `X` clear untuk `kTrackingAreaNone` (283–297); lain via `_drawDotFlatStyleWithColor:` (300).
- **Mouse** (406–436): pakai `[[self window] mouseLocationOutsideOfEventStream]` (408) bukan lokasi event; iterasi semua tag, match → set `selectedTag` (421), tulis balik ke `observer` via KVC (423–425), panggil `colorSelectorDidChange:` bila delegate respond (427–429).
- **Tracking areas** (206–234): dibuat sekali (guard `trackingAreas == nil`) dengan opsi `NSTrackingEnabledDuringMouseDrag | ...EnteredAndExited | ...ActiveInActiveApp | ...ActiveAlways` (213–216).
- **KVO lifecycle** (80–147, 444–457): self-registration `selectedTag` dijaga `observingSelfSelectedTag` (91, 449–451); bind/unbind buang observasi sebelumnya (100–107, 117–122); `dealloc` rujuk crash issue #2033 (441–442).
- `rectForColorViewAtIndex:` (152–179) `switch` tiap `case` berbagi satu body (164–176) — switch efektif dead/redundan.
- `observeValueForKeyPath:` register `context:nil` (90, 105) lalu bercabang `object == self`/`object == observer` (131, 134) — mengandalkan identitas objek.
- ⚠️ `_drawDotBevelStyleWithGradient:insideRect:` (305–339) didefinisikan/dideklarasikan tapi **tak dipanggil** di file (hanya `_drawDotFlatStyleWithColor:` dipakai di 300).

---

## 7. `SPComboPopupButton.m` (304)

Subclass `NSPopUpButton`: split tombol jadi area klik + area dropdown menu, gambar garis pemisah, cell kustom menangani tracking split.

### Ivar/konstanta
`.h:34–39` ivar `menuIsOpen`, `shouldDrawNonHighlightState`, `lineOffset`, `actionSelector`, `actionTarget`; property `.h:42,43`; synthesize 49–50. `#define kSPComboPopupButtonLineOffsetMini/Small/Regular` 16/21/23 (33–35); extension (37–41); private `@interface SPComboPopupButtonCell` (43–45).

### `#pragma mark`
`52/53 (Setup), 80/81 (Drawing), 148/149 (Click action overrides), 178/179 (Menu delegate implementation), 191, 222, 247/248 (Custom interaction handling)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 55–78 | | `initWithCoder:` / `initWithFrame:pullsDown:` / `cellClass` | Setup |
| 88 | 146 | `drawRect:` | Gambar tombol split, garis gradien pemisah |
| 151–176 | | `performClick:` / `target` / `setTarget:` / `action` / `setAction:` | Aksi klik |
| 181–189 | | `menuWillOpen:` / `menuDidClose:` | State menu |
| 193 | 218 | `_initCustomData` | Set line offset, delegate menu, migrasi target/action |
| 229–245 | | `drawTitle:withFrame:inView:` / `isHighlighted` (cell) | Cell kustom |
| 250 | 302 | `trackMouse:inRect:ofView:untilMouseUp:` (cell) | Split tracking: area klik vs dropdown |

### Temuan
- **Drawing** (88–146): `boundingLinePosition = maxX - lineOffset - 0.5` (91); saat `menuIsOpen` clip + gambar separuh kiri via `shouldDrawNonHighlightState = YES` (100–108) dan separuh kanan normal (111–117); else super draw (118–120). Pemisah = gradien CoreGraphics (`CGGradientCreateWithColors` 132), digambar ter-clip (134–139), CF object di-release (141–145).
- **Custom tracking** (250–302): bail ke super (`trackAsPerMenuButton`) bila bukan left-down (259–261), control view tanpa `lineOffset` (264–266), atau klik di kanan pemisah (269–271); else track dalam `activeRect` inset (278) toggle `setHighlighted:` (285–288), perform click via `performClick:` (295–296).
- **Click override** (151–156): `performClick:` bypass super, kirim langsung `actionSelector`/`actionTarget` — alasan target/action di-stash dari `super` di `_initCustomData` (214–217).
- `drawTitle:withFrame:inView:` (229–234) memutasi `frame.width` pakai `[controlView bounds]` (232) — **UNKNOWN** margin `-1`.
- `isHighlighted` (239–245) baca `shouldDrawNonHighlightState` dari control view (241) — coupling cell↔button.
- Private cell interface (43–45) tanpa body; `+cellClass` (75–78) mengembalikannya → tombol dari xib harus pilih kelas cell manual (komentar 72–73).

## Ringkasan lintas-file
- Tiga dari tujuh file merujuk simbol Swift-bridged via `#import "sequel-ace-Swift.h"` atau notifikasi `SPConstants`; file cell bergantung `SPLinkDrawState*` (`SPConstants.h:152–154`).
- **Pola link interaktif** dibagi tapi divergen: `SPTextAndLinkCell` region-test di `hitTestForEvent:` (169–188), sedangkan `SPActivityTextFieldCell` return trackable untuk seluruh sel (207–210) + rect test sendiri di `trackMouse:` (224).
- Hanya `SPColorSelectorView.m` (97–147, 444–457) + `SPTableView.m` (83–96, 327–338) mengelola registrasi/teardown observer; dua tipe cell tidak override `dealloc`.
