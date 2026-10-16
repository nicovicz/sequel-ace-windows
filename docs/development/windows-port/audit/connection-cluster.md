# Audit: Connection Swift cluster

Audit method-by-method tiga file klaster koneksi Swift. `Start`/`End` = nomor
baris di file masing-masing. Baseline commit `02c910932`.

File: `SAConnectionWindowController.swift` (773 baris),
`SAConnectionFormView.swift` (617), `SAConnectionService.swift` (617).

---

## File 1 — `SAConnectionWindowController.swift`

- Peran: `NSWindowController` standalone yang meng-host layar koneksi SwiftUI,
  me-resolve kredensial AWS IAM/Vault, connect via `SAConnectionService`, dan
  menyerahkan koneksi live ke tab dokumen baru.

### Tipe
`SAConnectionWindowController` (`@objc class : NSWindowController, SAConnectionDelegate, NSWindowDelegate`, 30–605);
`SAResolvedCredentials` (private struct nested, 289–292); `SACredentialFailure` (private struct Error, 294–297);
`SAConnectionInfoObjC` (extension, 609–678); `SAConnectionWindowView` (private struct View, 686–740);
`SAConnectionWindowController` (extension `SADatabaseDocumentProviding`, 746–773).

### `// MARK:`
`32 (Properties), 74 (Lifecycle), 128 (Connection screen), 460 (SAConnectionDelegate), 561 (Direct Connection via SAConnectionService), 607 (Populating a document's connection controller), 680 (The SwiftUI screen), 742 (SADatabaseDocumentProviding)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 35–72 | | `connectionService`, `formModel`, `favorites`, `selection`, `activeVaultLoginIdentifier`, `attempts`, `connectionHandedOff`, `viewCoordinator`, `placeholderSplitView`, `containerView`, `hostingView` | Properti |
| 76 | 79 | `init()` | Convenience init; buat window via `setupWindow` |
| 81 | 99 | `setupWindow()` | Bangun NSWindow, ukuran, judul, container/split view |
| 101 | 104 | `windowDidLoad()` | Panggil `installConnectionScreen` setelah load |
| 106 | 110 | `showWindow(_:)` | Install screen, show, set self sebagai window delegate |
| 114 | 126 | `windowWillClose(_:)` | Cancel koneksi/Vault login kecuali sudah handoff |
| 132 | 150 | `installConnectionScreen()` | Tukar placeholder jadi screen SwiftUI (idempoten) |
| 153 | 156 | `loadFavorites()` | Bangun `[SAFavoriteItem]` dari tree favorites |
| 159 | 178 | `applySelection(_:)` | Isi form dari seleksi sidebar; supersede attempt |
| 185 | 192 | `favoriteDictionary(withID:)` | Cari plist favorite by id ternormalisasi |
| 195 | 203 | `allFavorites(under:)` | Ratakan dict favorite lewat group (rekursif) |
| 212 | 270 | `connectUsingForm()` | Validasi, snapshot form, resolve kredensial, connect |
| 276 | 279 | `beginCredentialAttempt()` | Cancel Vault login, mulai attempt baru, return id |
| 282 | 286 | `cancelActiveVaultLogin()` | Cancel login Vault in-flight |
| 289–297 | | `SAResolvedCredentials`/`SACredentialFailure` | Tipe hasil/kegagalan kredensial |
| 300 | 328 | `resolveCredentials(for:_:)` | Dispatch resolusi kredensial by tipe koneksi |
| 337 | 365 | `ensureAWSDirectoryAuthorized()` | Prompt akses `~/.aws`, persist bookmark |
| 368 | 402 | `resolveAWSIAMToken(info:completion:)` | Generate token AWS IAM off main thread |
| 406 | 458 | `resolveVaultCredentials(info:completion:)` | Fetch kredensial Vault ephemeral off main queue |
| 462 | 514 | `connectionDidEstablish(_:info:)` | Handoff 8-langkah koneksi live ke dokumen baru |
| 516 | 521 | `connectionDidFail(withError:detail:)` | Hook kegagalan delegate; log saja |
| 529 | 545 | `warnIfRequestedSSLWasNotEstablished(_:info:)` | Warning saat SSL diminta tapi tak dipakai |
| 548 | 559 | `showConnectionError(title:detail:)` | Tampilkan alert warning sebagai sheet/modal |
| 566 | 604 | `connectDirectly(with:password:sshPassword:attemptID:)` | Panggil service; handoff atau lapor hasil |
| 623 | 677 | `apply(to:)` | Kebalikan `_buildConnectionInfo`; copy info ke controller |
| 688–739 | | `favorites`/`model`/`onSelect`/`onConnect`/`selection`/`searchQuery` (view), `body` (701–714), `sidebar` (716–739) | View SwiftUI |
| 748–772 | | `contentViewSplitter`, `databaseView()`, `parentWindowControllerWindow()`, `setConnection(_:)`, `isProcessing`, `updateWindowTitle(_:)` | `SADatabaseDocumentProviding` |

### Temuan / UNKNOWN
- **Konstruksi window** (81–99): content rect 860×540 (83), judul `"Connect to Server"` (88), min size 700×400 (90), `containerView` jadi content (91), placeholder split view (94–96). `windowDidLoad` + `showWindow` sama-sama panggil `installConnectionScreen`, idempoten via `hostingView == nil` (133).
- **`viewCoordinator`** (62) dideklarasikan + didokumentasikan (60–61) tapi **tak direferensikan** di file ini — dead state. **UNKNOWN** apakah dipakai file lain.
- **Conformance delegate**: `connectionDidFail` (516–521) hanya `NSLog`; komentar menyatakan jalur `connectDirectly` menampilkan alert-nya sendiri (518–520). **UNKNOWN** apakah `connectionDidFail` pernah dipanggil di flow standalone.
- **Identitas attempt ganda**: `connectUsingForm` (245) track id dari `attempts.begin() -> UInt`, diteruskan via `attempts.deliver(to:)` (247) ke `connectDirectly` (566) → `connectionService.connect` (567); sementara `SAConnectionService` punya attempt id `UInt64` sendiri (168–174). Dua fence independen.
- **Snapshot kredensial** (218–222): form di-snapshot sebelum leg Vault async agar edit berikutnya tak memasangkan kredensial dengan detail berbeda.
- **Cancel Vault login saat tutup** (114–126): `windowWillClose` panggil `beginCredentialAttempt()` (125) untuk **supersede** (bukan sekadar cancel) agar token login browser diinvalidasi (komentar 118–124).
- **Urutan handoff** (`connectionDidEstablish`, 462–514) dinomori: (1) window baru `tabManager?.newWindowForWindow()` (472); (2) `warnIfRequestedSSLWasNotEstablished` (481); (3) `info.apply(to: document.connectionController())` (488); (4) `connection.setDelegate(document)` (495); (5) clear token AWS IAM `connection.password = nil` (501–503); (6) `document.setConnection(connection)` (507); (7) `connectionHandedOff = true` (510); (8) `close()` (513).
- **Mapping error**: kegagalan kredensial → `showConnectionError` (252); kegagalan service → `errorMessage`/`errorDetail` join newline (594–597), title fallback `"Connection failed"` (599). `userCancelled` restore UI senyap (587–591); `databaseSelectionFailed` tetap handoff (576–581).

---

## File 2 — `SAConnectionFormView.swift`

- Peran: form editor SwiftUI untuk detail koneksi semua lima tipe, plus subview
  time-zone picker, color picker, dan optional-file row.

### Tipe
`SAConnectionFormView` (struct View, 31–409); `SATimeZonePicker` (private struct View, Equatable, 423–468);
`SAFavoriteColorPicker` (private struct View, 475–514); `SAOptionalFileRow` (private struct View, 521–617).

### `// MARK:`
`81 (Sections), 218 (Per-type field groups), 372 (Shared fields), 411 (Time-zone picker), 470 (Colour picker), 516 (Optional file row)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 33–50 | | `model`, `onConnect`, `validationFailure`, `isPresentingValidationFailure`, `roleFieldFocused` | Properti |
| 53 | 79 | `body` | `Form` terkelompok menyusun semua section + alert |
| 85–108 | | `typeSection`, `identitySection` | Pilih tipe (ganti tab bar XIB); nama + color picker |
| 110–216 | | `detailsSection`, `timeZoneSection`, `securitySection`, `sslFileSection`, `connectSection` | Section detail/TZ/security/SSL/connect |
| 220–370 | | `standardFields`, `socketFields`, `sshFields`, `awsIAMFields`, `vaultFields` | Field per tipe |
| 374–408 | | `databaseField`, `portField`, `optionalPrompt`, `namePrompt`, `submit` | Field bersama + submit (401–408) |
| 425–467 | | `choice`, `onSelect`, `==` (picker), `menu`, `body`, `selection` | Time-zone picker |
| 477–513 | | `colorIndex`, `noColorIndex`, `colors`, `body`, `colorLabel(at:)` | Color picker |
| 523–616 | | `title`, `kind`, `isEnabled`, `path`, `rejection`, `isPresentingRejection`, `body`, `chooseFile` | Optional file row |

### Temuan / UNKNOWN
- **Binding field** via `@ObservedObject model: SAConnectionFormModel` (33); bound fields `$model.info.<field>` (tipe 87, nama 103, host 222, user 225, password 228, socket 239, ssh host 255, aws profile 321, vault host 342). Switch visibilitas adalah property model: `showsSSLFileOptions` (60), `showsClearTextPluginToggle` (148), `showsSSLToggle` (154), `showsRequestServerPublicKeyToggle` (160), `forcesSSL` (168), `canAttemptConnection` (213).
- **Password AWS IAM** (310–315): `SecureField` disabled dengan binding kosong konstan + prompt "Generated from AWS IAM profile" (komentar 305–309).
- **Commit role Vault** (357–367): `@FocusState roleFieldFocused` (50) + `.focused` (363), `.onSubmit` (364), `.onChange(of: roleFieldFocused)` (365–367) — keduanya panggil `model.commitVaultCredentialsRole()`, mencerminkan `-controlTextDidEndEditing:`. Komentar: combo-box role list/Refresh ditunda ke C3 (354–356).
- **Desain alert validasi** (39–47, 66–78): `validationFailure` optional sengaja dipertahankan setelah dismiss dan hanya dibaca saat present; bool terpisah `isPresentingValidationFailure` agar SwiftUI bandingkan key-path (41–45).
- **Performa time-zone picker** (411–468): struct terpisah + `Equatable` eksplisit (428–430) + `menu` static cached (433); rasional closure-built `Binding` (465–467) tak pernah melewati batas view (413–422, 457–464).
- **Color picker** (470–514): sentinel `-1` (480), warna dari `SPFavoriteColorSupport` (483), label posisional (506–513).
- **Optional file row** (516–617): uncheck menghapus path (541–543); choose flow validasi via `SAConnectionFileValidator.rejection(forFileAt:kind:)` (593) + persist bookmark read-only security-scoped (607–613) sebelum simpan path (615); cancel bersihkan checkbox+path (585–589), mencocokkan `-chooseKeyLocation:`.

---

## File 3 — `SAConnectionService.swift`

- Peran: service standalone yang mengekstrak pembentukan koneksi MySQL dari
  `SPConnectionController`; menghasilkan `SPMySQLConnection` terkonfigurasi atau
  `SAConnectionResult` diagnostik.

### Tipe
`SAConnectionResult` (`@objc class`, 15–96); `SAConnectionPreferences` (`@objc class`, 99–123);
`SAConnectionService` (`@objc class`, 129–617).

### `// MARK:`
`223 (Public API), 332 (Private: MySQL Connection), 516 (Private: SSH Tunnel)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 16–34 | | `connection`, `sshTunnel`, `errorTitle`, `errorMessage`, `errorDetail`, `lastErrorID`, `rawErrorMessage`, `sshDebugMessages`, `connectionType`, `socketPath`, `databaseSelectionFailed`, `databaseSelectionError`, `userCancelled`, `isSuccess` | Field hasil |
| 36 | 51 | `init(connection:sshTunnel:databaseSelectionFailed:databaseSelectionError:)` | Init hasil sukses |
| 66 | 83 | `init(errorTitle:errorMessage:errorDetail:lastErrorID:rawErrorMessage:sshDebugMessages:connectionType:socketPath:)` | Init hasil gagal |
| 85 | 95 | `sshFailure(_:)` | Factory dari `SASSHTunnelFailure` |
| 100–104 | | `connectionTimeout`, `useKeepAlive`, `keepAliveInterval`, `enableQueryLogging`, `sslCipherList` | Preferensi |
| 106 | 122 | `fromUserDefaults()` | Bangun preferensi dari `UserDefaults`, buang cipher disabled |
| 131 | 131 | `SASSHTunnelCompletion` | Typealias callback tunnel |
| 134–166 | | `mySQLDelegate`, `stateLock`, accessor terkunci `activeTunnel`/`activeConnection`/`sshTunnelCompletion`/`cancelled`/`activeAttemptID` | State terkunci |
| 168 | 174 | `startAttempt()` | Naikkan attempt id, bersihkan cancelled, return id |
| 176 | 182 | `cancelAttempt()` | Invalidate attempt, set cancelled, drop completion |
| 184 | 188 | `isCurrentAttempt(_:)` | True bila id cocok + tak cancelled |
| 190 | 196 | `setActiveConnection(_:for:)` | Simpan connection hanya bila attempt current |
| 198 | 204 | `setActiveTunnel(_:for:)` | Simpan tunnel hanya bila attempt current |
| 206 | 212 | `setSSHTunnelCompletion(_:for:)` | Simpan completion hanya bila attempt current |
| 214 | 221 | `clearActiveState(for:)` | Nil connection/tunnel bila attempt current |
| 227 | 272 | `connectAWSIAM(with:info:preferences:region:profile:attemptID:sshPassword:parentWindow:completion:)` | Resolve token AWS di background lalu connect |
| 276 | 315 | `connect(with:preferences:password:sshPassword:parentWindow:completion:)` | Bangun tunnel (bila SSH) lalu MySQL, completion ter-fence |
| 318 | 330 | `cancel()` | Cancel attempt; disconnect connection + tunnel |
| 335 | 514 | `connectMySQL(info:preferences:password:tunnel:attemptID:completion:)` | Connect MySQL background, SSL/flag, select DB, timezone |
| 518 | 579 | `establishSSHTunnel(info:sshPassword:parentWindow:attemptID:completion:)` | Bangun + start `SPSSHTunnel`, wire callback state |
| 581 | 616 | `sshTunnelStateChanged(_:)` | Tangani state tunnel: connected, cancelled, failed |

### Temuan / UNKNOWN
- **Struct hasil** (15–96): init sukses 36–51, gagal 66–83, factory `sshFailure` 85–95; `isSuccess` butuh connection hidup, tanpa error title, tanpa DB-selection failure (34). `userCancelled` mutable di-set setelah konstruksi (32, dipakai 308). Ambiguitas izin local-network sengaja tak diputuskan di sini (53–55).
- **Ekstraksi preferensi** (106–122): timeout/keepalive/logging (109–112); buang cipher SSL disabled di marker regex `:?--` (113–119).
- **Fencing cancellation**: tiap setter state no-op kecuali attempt current (190–221); `connect` membungkus completion dalam `safeCompletion` yang menjadwalkan completion caller hanya saat current (287–291), dan fence yang sama dicek ulang setelah thread start (348), setelah `connect()` (417–421), setelah fallback retry (431), dan sebelum select DB (475).
- **Urutan `connect`** (276–315): attempt id baru (284); `.sshTunnel` bangun tunnel lalu `connectMySQL` dengan tunnel (293–299); tunnel gagal → `sshFailure` (301); tunnel cancel → hasil `userCancelled` (304–310); non-SSH langsung `connectMySQL` tunnel nil (313).
- **`connectMySQL`** (335–514): thread detached (343); socket vs tunnel vs TCP (353–373); set password (375–377); flag `allowDataLocalInfile`/`enableClearTextPlugin`/`requestServerPublicKey` (378–380); AWS IAM paksa cleartext + SSL (383–386); blok SSL dengan cert/key/CA + cipher list (388–402); buang kompresi (404–406); delegate + logging + timeout/keepalive (408–414); `conn.connect()` (416); fallback retry tunnel SSH via `localPortFallback()` (425–440); select DB (474–489); terapkan timezone (491–502); hasil sukses (511).
- **Mapping error**: gagal connect → title "Unable to connect" + detail khusus saat `lastErrorID == 1045` ("check username and password") (449–463); DB-selection gagal → hasil membawa connection + `databaseSelectionFailed`/`Error` (481–488). `SASSHTunnelFailure` → `sshFailure` (85–95); kegagalan MySQL pasca-tunnel menyimpan `sshDebugMessages` untuk klasifikasi port-forwarding (86–88).
- **Mapping error tunnel SSH**: port SSH invalid → failure pesan terlokalisasi (525–535); gagal buat tunnel → "Failed to create SSH tunnel" (547–549); state membedakan cancel (585–589), connected (594–597), terminal failure digerbangi `failureDiagnosticsReady` (598–613).
- **`connectAWSIAM`** (227–272): didokumentasikan jalur legacy (225); resolve token di background (240–250), gagal via `controller.failConnection(...)` saat token kosong/error (256–263), lalu defer password ke delegate bila berlaku (266–268) + `connect` (269–270).
- **Thread-safety** (136–166): `activeTunnel`, `activeConnection`, `sshTunnelCompletion`, `activeAttemptID`, `_cancelled` dijaga `stateLock`. **UNKNOWN** apakah ada aksesor lain di luar file ini yang melewati lock.
- **`cancel()`** (318–330): detach thread disconnect + nil state aktif; caller tak melihat completion karena attempt diinvalidasi (319).

---

## Ringkasan lintas-file
- `SAConnectionService.connect` hanya dikonsumsi `SAConnectionWindowController.connectDirectly` (567); `connectAWSIAM` (227) tak direferensikan dari dua controller ini — **UNKNOWN** caller (kemungkinan `SPConnectionController` legacy).
- `SAConnectionResult`/`SAConnectionPreferences` tipe batas; controller konsumsi `connection`, `databaseSelectionFailed`, `isSuccess`, `userCancelled`, `errorTitle`, `errorMessage`, `errorDetail` (576–601). Field diagnostik lain (`lastErrorID`, `sshDebugMessages`, `connectionType`, `socketPath`) **UNKNOWN** konsumennya.
- Identitas attempt dilacak dua kali: `UInt` di `SAConnectionAttemptSequence` (52, 245) dan `UInt64` di service (166). Keduanya harus current agar handoff terjadi.
