# Audit: `SPSSHTunnel.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Other/SSHTunnel/SPSSHTunnel.m`. Baseline commit `02c910932`.

- File: 1243 baris; header `SPSSHTunnel.h` 127 baris.
- Peran: controller ObjC yang memiliki `NSTask` tunnel `/usr/bin/ssh` — membangun
  argumen/env OpenSSH, memparse stderr ssh untuk state, menjalankan sheet prompt
  SSH_ASKPASS blocking, dan mengekspos permukaan `SPMySQLConnectionProxy` ke
  connection controller.
- Total: 42 definisi method + 1 fungsi C (`getRandomPort`).

## 1. Inventaris ivar/property + konstanta

### Ivar publik `.h:37–81`
`delegate` (37); `parentWindow`, `task`, `standardError`, `lastError`, `lastErrorLock`,
`tunnelConnectionName`, `tunnelConnectionVerifyHash`, `sshHost`, `sshLogin`, `remoteHost`,
`remoteSocketPath`, `password`, `keychainName`, `keychainAccount`, `requestedPassphrase`,
`identityFilePath`, `debugMessages`, `debugMessagesLock`, `sshPort`, `remotePort`, `localPort`,
`localPortFallback`, `connectionState` (39–61); `answerAvailableLock`, `currentKeyName` (63–64);
`stateChangeSelector` (66); `useHostFallback`, `connectionMuxingEnabled`, `requestedResponse`,
`passwordInKeychain`, `passwordPromptCancelled`, `taskExitedUnexpectedly` (68–73);
IBOutlet `sshQuestionDialog`, `sshQuestionText`, `sshPasswordKeychainCheckbox`, `sshPasswordDialog`,
`sshPasswordText`, `sshPasswordField` (75–80).

### Ivar class-extension privat `.m:50–76`
`tunnelConnection` (`NSConnection *`, 55 — privat agar supresi deprecation tak ter-emit ulang);
`authService` (`SASSHTunnelAuthService *`, 61); `promptCoordinator` (`SASSHTunnelPromptCoordinator *`, 66);
`transport` (`SASSHTunnelTransport`, 72); `socketServer` (`SASSHTunnelSocketServer *`, 73);
`standardErrorDrainCoordinator` (`SASSHStderrDrainCoordinator *`, 75).

### Property `.h:83–94` & synthesize `.m:88–90`
readonly `passwordPromptCancelled` (83), `taskExitedUnexpectedly` (84), `failureDiagnosticsReady` (85–86),
`connectionAttemptPending` (87–88); readwrite IBOutlet `sshQuestionText`/`sshQuestionDialog`/`sshPasswordDialog`/
`sshPasswordText`/`sshPasswordField` (90–94). **Catatan:** `sshPasswordKeychainCheckbox` (`.h:77`) **tanpa** synthesize/property — IBOutlet ivar polos.

**Konstanta file-local: tidak ada** `#define`/`static const`. Satu deklarasi file-local: `static unsigned short getRandomPort(void);` (48; body 1225–1243). Semua key pref bersifat **extern** (`SPSSHEnableMuxingPreference` 123, `SPSSHClientPath` 462, `SPConnectionTimeoutValue` 434/910, `SPUseKeepAlive` 435, `SPKeepAliveInterval` 436, `SPSSHConfigContainsUserKnownHostsFile` 521, `SPSSHUsualKnownHostsFile` 522, `SPSSHDefaultKnownHostsFile` 548, `SPSSHConfigFile` 579/583).

## 2. Semua `#pragma mark`

`.m:941 (Connection surface), 966 (SASSHTunnelAuthSource), 1200, 1223`.
Anchor struktur: `@interface SPSSHTunnel ()` 50, `@end` 84, `@implementation` 86, `@end` 1221.

## 3. Tabel method (urut baris)

### Init, setter konfigurasi, permukaan state/error (97–387)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 97 | 182 | `initToHost:port:login:tunnellingToPort:onHost:` | Simpan detail koneksi; setup DO connection, auth service, transport |
| 184 | 187 | `failureDiagnosticsReady` | Teruskan flag dari drain coordinator |
| 189 | 192 | `connectionAttemptPending` | Teruskan flag queued-reconnect dari drain coordinator |
| 198 | 204 | `setConnectionStateChangeSelector:delegate:` | Catat delegate + selector callback state |
| 209 | 221 | `setParentWindow:` | Simpan parent window + muat nib SSHQuestionDialog |
| 227 | 233 | `setPassword:` | Simpan password lokal kecuali keychain dipakai |
| 238 | 244 | `setKeyFilePath:` | Expand tilde, verifikasi identity file ada, simpan path |
| 246 | 249 | `setRemoteSocketPath:` | Simpan path unix-socket remote untuk forwarding |
| 255 | 278 | `setPasswordKeychainName:account:` | Simpan nama/account item keychain untuk lookup on-demand |
| 283 | 294 | `state` | State proxy; laporkan WaitingForAuth saat prompt auth aktif |
| 299 | 306 | `lastError` | Salinan terkunci thread-safe last error |
| 308 | 313 | `setLastError:` | Setter terkunci thread-safe |
| 321 | 327 | `addDebugMessage:` | Append terkunci satu baris ke buffer debug |
| 329 | 334 | `debugMessages` | Join buffer debug dengan newline (terkunci) |
| 339 | 359 | `connect` | Entry; gate pada state + drain coordinator, lalu mulai attempt |
| 361 | 375 | `startConnectionAttempt` | Reset debug/prompt state + spawn thread tugas SSH |
| 377 | 387 | `completeStandardErrorDrain` | Notifikasi delegate, lalu mulai attempt yang di-queue selama drain |

### Task lifecycle & stderr (394–939)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 394 | 776 | `launchTask:` | Bangun argumen/env OpenSSH, launch task, drain stderr, teardown |
| 781 | 785 | `disconnect` | Disconnect publik; tak mempertahankan queued reconnect |
| 787 | 791 | `disconnectForReconnect` | Varian disconnect mempertahankan queued reconnect |
| 793 | 825 | `disconnectPreservingQueuedReconnect:` | Cancel prompt, opsional cancel attempt, abort task jalan |
| 830 | 842 | `abortTask` | Cancel prompt + terminate `NSTask` jalan |
| 848 | 921 | `standardErrorHandler:` | Parse baris stderr ssh; transisi state; re-arm baca pipe |
| 926 | 929 | `localPort` | Port tunnel lokal yang dialokasikan |
| 934 | 939 | `localPortFallback` | Port lokal fallback, atau 0 bila tak dipakai |

### Connection surface (941)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 951 | 954 | `getPasswordWithVerificationHash:` | Teruskan permintaan password ke `authService` |
| 956 | 959 | `getResponseForQuestion:` | Teruskan pertanyaan yes/no ke `authService` |
| 961 | 964 | `getPasswordForQuery:verificationHash:` | Teruskan query password ke `authService` |

### SASSHTunnelAuthSource (966)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 968 | 971 | `verificationHash` | Kembalikan token verify hash koneksi |
| 973 | 976 | `heldPassword` | Password yang disimpan lokal |
| 978 | 981 | `usesKeychainPassword` | Apakah keychain memasok password |
| 983 | 986 | `keychainItemName` | Nama item keychain tersimpan |
| 988 | 991 | `keychainItemAccount` | Account item keychain tersimpan |
| 998 | 1017 | `promptForResponseToQuestion:` | Blok worker thread sementara sheet yes/no dijawab |
| 1019 | 1049 | `workerGetResponseForQuestion:` | Setup/jalankan sheet pertanyaan modal di main thread |
| 1054 | 1062 | `closeSSHQuestionSheet:` | Akhiri sesi modal pertanyaan; catat jawaban yes/no |
| 1069 | 1092 | `promptForPasswordForQuery:` | Blok worker thread sementara sheet password dijawab |
| 1094 | 1139 | `workerGetPasswordForQuery:` | Setup/jalankan sheet password modal di main thread |
| 1144 | 1174 | `closeSSHPasswordSheet:` | Akhiri sesi modal password; simpan jawaban, opsional simpan keychain |
| 1186 | 1189 | `cancelPendingPrompt` | Minta prompt coordinator dismiss prompt in-flight |
| 1194 | 1197 | `prepareFilePathForSshCommand:` | Decode percent-escape di path untuk ssh |

### Teardown (1200)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1202 | 1219 | `dealloc` | Invalidasi prompt coordinator/connection/socket, disconnect, bersihkan error |

### Fungsi C (1223)
| Start | End | Function | Tujuan |
| --- | --- | --- | --- |
| 1225 | 1243 | `getRandomPort(void)` | Bind socket ephemeral untuk cari port TCP lokal bebas |

> Catatan: `abortTask` (830) ditulis `-(void)` tanpa spasi sehingga anchor `^[-+] (` melewatkannya.

## 4. Temuan menonjol / UNKNOWN

**Argumen OpenSSH** (semua di `launchTask:`, 394–776):
- `-v` (486) verbose agar stderr bisa diparse; `-N` (489) tunnel-only.
- Muxing ON (493–503): `-o ControlMaster=auto`, `-o ControlPath=<tmpdir>/SPSSH-<8 hex>` (path = `login@host:port`, SHA-256 8 char pertama, 500–502). Muxing OFF (504–508): `-S none`, `-o ControlMaster=no`.
- Selalu: `-o ExitOnForwardFailure=yes` (511), `-o ConnectTimeout=<n>` bila timeout>0 (514–516), `-o NumberOfPasswordPrompts=3` (519).
- Known hosts: kecuali `SPSSHConfigContainsUserKnownHostsFile`, tambah `-o UserKnownHostsFile="<path>"` (570); fallback sandbox `SPSSHDefaultKnownHostsFile` (546–555).
- Config: `-F <sshConfigFile>` (586), default bundled `SPSSHConfigFile` (582–584). Identity `-i <identityFilePath>` bila diset (593–595).
- Keepalive (598–603): `TCPKeepAlive=yes`, `ServerAliveInterval=ceil(keepAliveInterval)`, `ServerAliveCountMax=3`, `ConnectTimeout=10`.
- Target: `-p <sshPort>` bila diset (606–608); `login@host` atau `host` (609–614).
- Forwarding (615–624): socket path via `-L <localPort>:<remoteSocketPath>`; host-fallback mengemit **dua** `-L`; else satu `-L`.

**Environment** (`taskEnvironment`, 628–654): mulai dari env `NSProcessInfo` (630), hapus `SHELL` agar ProxyJump tetap sandbox (632–633); set `SSH_ASKPASS=<SequelAceTunnelAssistant>` (635), `DISPLAY=:0` (636), `SP_CONNECTION_NAME` (637), `SP_CONNECTION_VERIFY_HASH` (638), `SP_SSH_TUNNEL_TRANSPORT` (639), socket path env (640–642); `SP_PASSWORD_METHOD` = keychain/UI/none (643–653). Komentar: nama/account keychain sengaja tak dimasukkan env (644–647).

**Prasyarat SSH_ASKPASS** (672–714): cek `/dev/tty` sekali; komentar panjang (687–710) menjelaskan DISPLAY + tanpa controlling terminal diperlukan agar OpenSSH pakai SSH_ASKPASS, dan `NSTask` tak bisa menghindari mewarisi control terminal.

**Parsing stderr** (`standardErrorHandler:`, 848–921): decode ASCII (857), split `\n` (860), trim (862). Trigger substring: `"Local forwarding listening on"` atau `"mux_client_request_session: master session id: "` → `SPMySQLProxyConnected` (869–870); `"Connection established"` → `SPMySQLProxyWaitingForAuth` (876–877); `"bind: Address already in use"` → drain + Idle + abort (883–888); `"closed by remote host."` (890–895); `"Permission denied ("`/`"No more authentication methods to try"` (896–901); `"connect failed: Connection refused"` → `SPMySQLProxyForwardingFailed` tanpa drain (902–905); `"Operation timed out"` (906–911). Pipe re-arm hanya bila `recordStandardErrorReadWithByteCount:` true; baca kosong = EOF (918–920).

**Transisi state**: Idle→Connecting (421); launch gagal → `SPMySQLProxyLaunchFailed` (727); exit tak terduga → Idle + `taskExitedUnexpectedly=YES` (750–755). State terminal sengaja dipublikasikan sekali setelah drain (881–882, 765–771).

**Prompt/password**: worker thread blok pada `answerAvailableLock` lalu lock kedua (998–1017, 1069–1092); worker main-thread (1019, 1094) present `beginSheet:` + `runModalForWindow:`. Dismissal closure didaftarkan menjawab "no"/nil saat teardown (1039–1046, 1128–1136). Nama passphrase key via regex `^\s*Enter passphrase for key '(.*)':\s*$` (1107); simpan keychain saat checkbox bila key name diketahui (1164–1168). `closeSSHPasswordSheet:` set `passwordPromptCancelled` saat tak ada passphrase (1171).

**UNKNOWN**
- `failureDiagnosticsReady`/`connectionAttemptPending` adalah flag delegate-facing tapi file ini tak pernah membacanya (consumen di luar pasangan yang diaudit).
- Set/urutan persis tipe Swift `SASSHTunnelAuthService`, `SASSHTunnelPromptCoordinator`, `SASSHStderrDrainCoordinator`, `SASSHTunnelTransportSelection`, `SAKeychainAccess`.
- Ivar `requestedResponse` ditulis oleh `closeSSHQuestionSheet:` (1056) **dan** `closeSSHPasswordSheet:` (1146) — kenapa callback password menimpa flag jawaban pertanyaan (kemungkinan shared-state smell).
- `prepareFilePathForSshCommand:` dinamai "escape spaces and special characters" (1191–1193) tapi hanya `stringByRemovingPercentEncoding` (1196) — misnomer.
- `abortTask` mengomentari `task = nil;` (840) + delegate clearing (814–817) — perilaku retained-task disengaja, rasional tak dinyatakan.
- Dua anotasi `// TODO: leaks` pada `waitForDataInBackgroundAndNotify` (670, 919) dan `waitUntilExit` (724).
- `[task SPlaunch]`/`[task SPterminate]` (718, 837) adalah kategori `NSTask` dari `SPThreadAdditions.h`/`SPFileHandle.h`.
