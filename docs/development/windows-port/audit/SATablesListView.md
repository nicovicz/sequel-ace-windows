# Audit: `SATablesListView.swift`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Views/TableViews/SATablesListView.swift`. Baseline commit `02c910932`.

- File: 900 baris.
- Peran: list sidebar tabel & view; mengganti type-select AppKit dengan fuzzy
  type-ahead (prefix → substring → subsequence), merutekan keystroke lewat
  `NSTextInputClient` agar IME berkomposisi normal, menunda commit baris yang
  cocok sampai urutan load selesai, dan menampilkan badge feedback pencarian
  transparan.
- Total: 61 definisi (termasuk property).

## 1. Tipe & konstanta

| Tipe | Jenis | Baris |
| --- | --- | --- |
| `SATablesListView` | `@objc final class : SPTableView, NSTextInputClient` | 20–891 |
| `SARowEntry` | `private struct : Equatable` | 854–860 |
| `SAClickThroughVisualEffectView` | `@objc private final class : NSVisualEffectView` | 895–900 |

Konstanta/timer: `typeAheadResetInterval = 0.3` (22), settle timer 0.3 s (381),
`feedbackHideTimer` 1.2 s (751), fade `NSAnimationContext.duration` 0.25 s (764),
badge `heightAnchor` 22pt (825), insets 8/4/−8/−16 (820–828),
`feedbackVisibilityGeneration` (77, 742).

Kolaborator eksternal (lokasi **UNKNOWN** di sini): `SATypeAheadMatcher` (24),
`SATypeAheadSelectionLoad<SPDatabaseDocument>` (32), `SAMarkedTextInputState` (58),
`NotificationToken`/`NotificationCenter.observe` (37–49).

## 2. Semua `// MARK:`

`551 (NSTextInputClient), 732 (Search feedback overlay)`.

## 3. Tabel definisi (urut baris)

### Properti tersimpan
| Start | Name | Tujuan |
| --- | --- | --- |
| 22 | `typeAheadResetInterval` | Jendela 0.3 s akumulasi keystroke type-ahead |
| 24 | `typeAhead` | Instance matcher fuzzy |
| 30 | `pendingSelection` | Baris cocok + identitas, menunggu settle |
| 31 | `pendingSelectionTimer` | Timer settle sebelum commit |
| 32 | `selectionLoad` | Lacak apakah load seleksi ter-commit berjalan |
| 37–39 | `windowDeactivationSubscription`, `menuTrackingSubscription`, `windowMouseDownMonitor` | Observer pembatal |
| 44 | `scrollInteractionSubscriptions` | Observer scroll user |
| 49 | `documentTaskSubscriptions` | Observer task dokumen |
| 50–54 | `isCommittingPendingSelection`, `documentWithTaskStartedDuringPendingSelection`, `pendingSelectionRetryDocument`, `deferredRowCommand`, `deferredRowCommandDocument` | State commit tertunda |
| 58–67 | `markedTextState`, `compositionExtendsSearch`, `routedInputEvent`, `routedInputWasConsumed`, `routedInputCallbackReceived`, `isDiscardingComposition` | State IME/composition |
| 70–77 | `lastSearchMatched`, `feedbackOverlay`, `feedbackLabel`, `feedbackHideTimer`, `feedbackVisibilityGeneration` | Badge |

### Lifecycle & override respons
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 79 | 101 | `awakeFromNib()` | Matikan type-select bawaan, subscribe notifikasi task |
| 103 | 166 | `viewWillMove(toWindow:)` | Bangun ulang observer window/scroll saat view pindah |
| 168 | 171 | `deinit` | Hapus badge + monitor mouse |
| 193 | 235 | `keyDown(with:)` | Rutekan input teks; tunda/commit row command; Escape cancel |
| 251 | 256 | `mouseDown(with:)` | Cancel match tertunda sebelum teruskan klik |
| 258 | 261 | `rightMouseDown(with:)` | Cancel match tertunda sebelum klik kanan |
| 263 | 268 | `resignFirstResponder()` | Cancel search saat fokus hilang |

### Core type-ahead (privat)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 173 | 179 | `removeWindowMouseDownMonitor()` | Hapus + nil monitor mouse-down lokal |
| 181 | 191 | `cancelTypeAheadIfNeeded()` | Cancel hanya bila ada state type-ahead aktif |
| 240 | 249 | `isRowCommandBlockedByLoading(_:)` | True untuk key edit/move yang ditolak saat load |
| 270 | 280 | `cancelTypeAhead()` | Teardown semua state search/composition tertunda |
| 283 | 292 | `resolveTypeAhead()` | Akhiri search, commit baris cocok segera |
| 294 | 299 | `captureDocumentTaskStartedDuringPendingSelection(_:)` | Catat dokumen yang load-nya dipicu seleksi |
| 301 | 344 | `documentTaskEnded(_:)` | Retry pending selection / replay row command tertunda |
| 346 | 354 | `replayDeferredRowCommand()` | Kirim ulang event key tertunda ke tabel |
| 356 | 359 | `cancelDeferredRowCommand()` | Bersihkan row command tertunda + dokumennya |
| 363 | 366 | `suspendPendingCommit()` | Invalidate timer settle tanpa lupa match |
| 369 | 384 | `schedulePendingCommit()` | Restart timer settle untuk match aktif |
| 388 | 400 | `discardComposition()` | Buang teks IME setengah jadi via input context |
| 406 | 429 | `routeThroughInputContext(_:)` | Tawarkan key ke input context, kembalikan konsumsi |
| 432 | 443 | `isTextEntry(_ event:)` | True bila event tampak teks biasa, bukan shortcut |
| 450 | 488 | `performTypeAhead(_:atTime:)` | Perluas search dengan teks commit, tampilkan match terbaik |
| 491 | 493 | `commitPendingSelection()` | Resolve seleksi via `selectionLoad` |
| 495 | 549 | `selectPendingMatch()` | Pilih baris cocok lewat gate delegate, lacak load |
| 837 | 847 | `isSearchableScalar(_:)` | Tolak control char + scalar fungsi-key |
| 866 | 890 | `searchableRows()` | Bangun identitas baris dari metadata group, bukan seleksi |

### `NSTextInputClient` (551)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 555 | 574 | `insertText(_:replacementRange:)` | Teks commit (keystroke/IME) perluas search |
| 578 | 597 | `doCommand(by:)` | Jalankan command editing IME di buffer komposisi |
| 601 | 613 | `setMarkedText(_:selectedRange:replacementRange:)` | Tampilkan teks composing di badge tanpa search |
| 615 | 650 | `updateCompositionPresentation(wasComposing:)` | Update badge/timer saat komposisi mulai/berakhir |
| 654 | 672 | `unmarkText()` | Terima teks composed + cari |
| 677 | 679 | `keepSearchAliveDuringComposition()` | Refresh liveness matcher selama komposisi |
| 684 | 686 | `inputEventTime` (var) | Timestamp teks commit: event ter-rutekan atau now |
| 688 | 690 | `noteInputCallback()` | Tandai callback client menjawab event |
| 692–702 | | `selectedRange()`/`markedRange()`/`hasMarkedText()` | State buffer komposisi |
| 704 | 706 | `attributedSubstring(forProposedRange:actualRange:)` | Delegasi substring ke buffer komposisi |
| 708 | 710 | `validAttributesForMarkedText()` | Array atribut kosong |
| 714 | 726 | `firstRect(forCharacterRange:actualRange:)` | Anchor window kandidat IME ke badge/dasar list |
| 728 | 730 | `characterIndex(for:)` | Selalu `NSNotFound` |

### Feedback overlay (732)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 737 | 754 | `showSearchFeedback(_:matched:)` | Pin badge dengan teks, warna merah bila tak match, arm hide |
| 756 | 772 | `hideSearchFeedback()` | Fade badge, hormati guard generasi |
| 774 | 781 | `removeFeedbackOverlay()` | Teardown view badge + timer |
| 785 | 835 | `ensureFeedbackOverlay()` | Bangun/pin badge di atas scroll view (lazy) |

### `SARowEntry` (854–860)
`title` (857), `section` (858), `objectType` (859).

### `SAClickThroughVisualEffectView` (895–900)
`hitTest(_:)` (897–899) — return nil agar badge tak menangkap klik.

## 4. Temuan menonjol / UNKNOWN

- **Fuzzy type-ahead (bukan type-select AppKit).** `allowsTypeSelect = false` (82); matching delegasi ke `SATypeAheadMatcher` via `bestMatch(appending:candidates:atTime:)` (466). Klaim urutan prefix→substring→subsequence di doc (13–15) tapi matcher **UNKNOWN**.
- **IME/marked text.** Teks komposisi ditampilkan tapi **tak** di-search sampai commit (599–600). `setMarkedText` simpan ke `markedTextState` + update presentasi saja (604–607); search hanya lihat teks dari `insertText`/`unmarkText` (569, 667).
- **Commit tertunda (settle).** Seleksi sinkron memulai load, jadi match di-commit via timer (376–381). Commit di-*suspend* selama komposisi (363–366, 633).
- **Gate row command tertunda.** Key saat tabel terpilih sedang load diparkir di `deferredRowCommand` (228–230), di-replay saat task berakhir + first-responder masih (347–353). Key yang digerbangi `isRowCommandBlockedByLoading`: Return/keypad-Enter, Home/PageUp/End/PageDown, arrow (242–244).
- **Gate delegate pada seleksi programatik.** `selectPendingMatch` panggil `selectionShouldChange?(in:)` + `shouldSelectRow` eksplisit (521–522); saat ditolak sementara dokumen working, simpan `pendingSelectionRetryDocument` untuk retry akhir task (523–531).
- **Identitas vs index.** Match menyimpan `SARowEntry` (title+section+objectType) karena reload bisa mengurut ulang baris; seleksi di-relokasi by identitas; tak ada seleksi bila hilang (500–515, 849–859).
- **Badge.** Merah bila tak match, `labelColor` sebaliknya (745); auto-hide setelah 1.2 s (751). `feedbackVisibilityGeneration` melindungi dari fade-out telat menyembunyikan badge yang muncul ulang (75–77, 761, 767). Click-through by design (893–899).
- **Detail verdict routing.** `routeThroughInputContext` memprioritaskan verdict callback client bila ada, else boolean input-context (423–428) — penting agar spasi leading yang ditolak tetap sampai ke tabel (458–463).
- **UNKNOWN**: semantik persis `SATypeAheadMatcher`/`SATypeAheadSelectionLoad`/`SAMarkedTextInputState`; definisi `NotificationToken`; perilaku `SPTableView` base yang diandalkan via `super.*` (211–234, 251–268, 353, 580).
