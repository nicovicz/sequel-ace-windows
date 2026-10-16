# Audit: AWS cluster

Audit method-by-method klaster AWS. `Start`/`End` = nomor baris di file masing-masing.
Baseline commit `02c910932`. (`AWSIAMAuthManager.swift` sudah diaudit terpisah.)

File: `AWSCredentials.swift` (479), `AWSDirectoryBookmarkManager.swift` (391),
`AWSLoginCredentialsProvider.swift` (307), `AWSSSOClient.swift` (380),
`AWSSTSClient.swift` (577), `RDSIAMAuthentication.swift` (366), `SAAWSLoginRefresh.swift` (525).

---

## 1. `AWSCredentials.swift` (479)

Parse `~/.aws/credentials` + `~/.aws/config` (INI) jadi config key/value, resolusi rantai profile
(`source_profile`, `sso-session`), dan model kredensial.

Tipe: `AWSCredentialsError` (34–55), `AWSCredentials` (58–451), extension ObjC (455–479).
Konstanta: `log` (84); error case 35–39; literal `"default"` (119, 232, 360, 476).
`// MARK:` `60 (Properties), 86 (Initialization), 146 (Validation), 173 (File Paths), 206 (Profile Loading), 445 (Description), 453 (Objective-C Compatibility)`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 41–54 | | `errorDescription` | Teks terlokalisasi per case |
| 62–84 | | `accessKeyId`/`secretAccessKey`/`sessionToken`/`expiration`/`profileName`/`roleArn`/`mfaSerial`/`sourceProfile`/`region`/`ssoSession`/`ssoStartURL`/`ssoRegion`/`ssoAccountID`/`ssoRoleName`/`loginSession`/`log` | Field |
| 89–144 | | `convenience init(...sessionToken:)` / `init(...expiration:)` / `init(profile:)` | Init |
| 148–171 | | `isValid` / `requiresMFA` / `requiresRoleAssumption` / `isSSOProfile` / `isLoginProfile` | Validasi |
| 175–204 | | `credentialsFilePath` / `configFilePath` / `credentialsFileExists` / `isAWSDirectoryAuthorized` | Jalur file |
| 210 | 238 | `availableProfiles()` | Union profile, `default` pertama |
| 244 | 325 | `loadProfileConfiguration(for:visited:)` | Merge creds+config, rantai, siklus |
| 328–350 | | `loadSSOSessionConfiguration(named:)` / `readAWSFileContents(at:)` | Baca sso-session + file |
| 353 | 402 | `parseAWSFile(_:forProfile:isConfigFile:)` | Parser INI section/key-value |
| 405–443 | | `addProfiles(from:to:isConfigFile:)` / `profileNames(inFileContents:isConfigFile:)` | Ekstrak nama profile |
| 447–478 | | `description` / `credentials(withProfile:error:)` / `profileConfiguration(forProfile:)` | Debug + bridge ObjC |

**Temuan**: credentials menang atas config saat bentrok; config hanya mengisi key absen (281, 290–292). `source_profile` rekursif dengan guard siklus `visited` (244–251, 307–318). `sso_session` diambil dari `[sso-session X]` via `parseAWSFile(..., isConfigFile:false)` (298–304, 332). Section config terima `NAME` dan `profile NAME` (359–362); `[sso-session …]` dikecualikan dari nama selectable (429–430). **UNKNOWN**: apakah override `AWS_SHARED_CREDENTIALS_FILE` diharapkan di luar scope bookmark (hanya default path digerbangi 264).

---

## 2. `AWSDirectoryBookmarkManager.swift` (391)

Kelola akses security-scoped bookmark ke `~/.aws` untuk baca/tulis sandbox-compatible, dengan start/stop ber-refcount.

Tipe: `AWSDirectoryBookmarkManager` (35–385), `Notification.Name` extension (389–391).
Konstanta: `bookmarkCreationOptions` (42–44), `awsDirectoryPath` (61–63), notifikasi `AWSDirectoryAuthorizationChanged` (390).
`// MARK:` `58 (Public API), 73 (Debug/Testing Options), 304 (File Reading), 387`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 37–56 | | `shared` / `log` / `bookmarkCreationOptions` / `stateLock` / `isAccessingAWSDirectory` / `accessReferenceCount` / `resolvedAWSDirectoryURL` / `init()` | State |
| 61–71 | | `awsDirectoryPath` / `awsDirectoryBasePath` | Path |
| 80–134 | | `isAWSDirectoryAuthorized` / `hasAWSDirectoryBookmark()` / `revokeAllAWSDirectoryBookmarks()` | Otorisasi |
| 139–211 | | `startAccessingAWSDirectory()` / `stopAccessingAWSDirectory()` | Akses ber-refcount |
| 215–276 | | `addAWSDirectoryBookmark(from:)` / `revokeAWSDirectoryBookmark()` / `replaceAWSDirectoryBookmark(with:)` | Bookmark |
| 279–302 | | `isAWSDirectoryURL(_:)` / `authorizedAWSDirectoryURL` / `isAWSDirectoryBookmarkKey(_:)` | Helper |
| 308–384 | | `readAWSFileContents(at:)` / `awsFileExists(at:)` / `resolvedAWSPath(for:)` / `resolvePathForCurrentAWSDirectory(_:)` / `currentResolvedAWSDirectoryURL()` / `stopAllAccessingAWSDirectory()` | Baca file + path |
| 390 | 390 | `AWSDirectoryAuthorizationChanged` | Nama notifikasi |

**Temuan**: key bookmark dinormalisasi URL-decode + backslash→slash + uji suffix/last-component `/.aws` (290–302). Refcount; `stop` hanya panggil `stopAccessingSecurityScopedResource` saat 1→0 (190–208). `isAWSDirectoryAuthorized` self-heal revoke stale setelah start gagal (89–98) + terima baca filesystem langsung untuk non-sandbox (100–105). **UNKNOWN**: apakah `SecureBookmarkManager.bookmarkFor(filename:)` sendiri memanggil `startAccessingSecurityScopedResource` — `startAccessingAWSDirectory` mengasumsikan URL usable (156–162).

---

## 3. `AWSLoginCredentialsProvider.swift` (307)

Resolve + renew kredensial console sign-in `aws login` dari `~/.aws/login/cache`, tulis sesi renewed secara atomik.

Tipe: `AWSLoginAuthError` (35–53), provider (60–281), extension (285–307).
Konstanta: `refreshLock` (63), `minimumFallbackLifetime = 60` (66), `refreshTransport` (69), suffix `/login/cache` + env `AWS_LOGIN_CACHE_DIRECTORY` (218–221), nama cache `sha256Hex(loginSession)+".json"` (230–231).
`// MARK:` `112 (Renewal), 214 (Cache Location), 234 (Cache Parsing), 241 (File Reading), 270 (Helpers), 283`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 41–110 | | `errorDescription` / `log` / `refreshLock` / `minimumFallbackLifetime` / `refreshTransport` / `resolveCredentials(for:)` / `resolvesThroughConsoleSignIn(_:)` / `needsWriteAccessGrant(for:)` | Resolve |
| 117–212 | | `renewCredentials(cachedAt:fallbackRegion:)` / `refresh(_:cachedAt:fallbackRegion:)` / `loadSession(_:)` | Renewal + atomic write |
| 217–268 | | `cacheDirectory` / `cacheFilePath(forLoginSession:)` / `cacheFileName(forLoginSession:)` / `parseCachedCredentials(fromJSON:now:)` / `readFileContents(at:)` / `withCacheFileAccess(_:_:)` | Cache |
| 273–306 | | `sha256Hex(_:)` / `resolveCredentialsObjC(for:error:)` | Helper + ObjC |

**Temuan**: wajib `login_session` (74–76). Refresh hanya bila `needsRefresh` (82–84). Double-check renewal: baca ulang sesi on-disk di bawah lock, return bila sudah refresh (118–122). Gagal refresh: prefer kredensial on-disk lebih baru, else cache dengan >60 s sisa, else rethrow (124–140). Renewal pakai DPoP key + client id + refresh token (145–149); region dari `issuingRegion` else profile (151–155). Atomic replacement cek refresh token on-disk (173–182). `loadSession` retry `Thread.sleep(0.1)` (206–208). **UNKNOWN**: apakah 0.1 s cukup di semua writer (206).

---

## 4. `AWSSSOClient.swift` (380)

Resolve kredensial IAM Identity Center lewat token bearer cache + SSO Portal `GetRoleCredentials`.

Tipe: `AWSSSOClientError` (35–62), `AWSSSOClient` (68–380).
Konstanta: `requestTimeout = 30` (71), `defaultDNSSuffix = "amazonaws.com"` (72), `/sso/cache` (119), path `.federation/credentials` + query `account_id`/`role_name` (175–179), header bearer `x-amz-sso_bearer_token` (191).
`// MARK:` `74 (Resolution Async), 115 (Token Cache), 153 (Portal Endpoint), 162 (GetRoleCredentials), 247 (Synchronous Wrapper), 313 (File Reading), 330 (Helpers)`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 44–113 | | `errorDescription` / `log` / `requestTimeout` / `defaultDNSSuffix` / `resolveCredentials(for:cacheDirectory:session:)` | Orchestrasi |
| 118–151 | | `tokenCacheDirectory` / `tokenCacheKey(for:)` / `cacheFileName(forKey:)` / `parseAccessToken(fromJSON:now:)` | Cache token |
| 156–245 | | `portalHost(forRegion:)` / `fetchRoleCredentials(...)` / `parseRoleCredentials(fromJSON:)` | Portal |
| 253–379 | | `resolveCredentialsSynchronously(for:)` / `resolveCredentialsObjC` / `nsError` / `readFileContents` / `resolvedRegion` / `expirationDate` / `parseISO8601Date` / `sha1Hex` | Wrapper + helper |

**Temuan**: key cache `sso_session` lalu `sso_start_url` (124–132); nama file hash SHA-1 (135–137). `parseAccessToken` wajib `accessToken` non-kosong + tolak `expiresAt` kedaluwarsa (140–148). `+` di query → `%2B` (182). Status: 200 parse, 401 `tokenExpired`, 403 `accessDenied`, else `invalidResponse` (211–221). Wrapper sync assert bukan main-thread + timeout `requestTimeout+5` (253–275). **UNKNOWN/notable**: tanpa refresh token/DPoP — bergantung `aws sso login` eksternal; token kedaluwarsa → `tokenExpired` (147, 215).

---

## 5. `AWSSTSClient.swift` (577)

Klien STS yang SigV4-sign `AssumeRole` (opsional MFA), parse XML, + wrapper ObjC sinkron.

Tipe: `AWSSTSClientError` (35–62), `AWSSTSClient` (65–497), `STSResponseParser` (502–577).
Konstanta: `algorithm` (69), `service` (70), `awsRequest` (71), `defaultSessionDuration = 3600` (72), `requestTimeout = 30` (73), `defaultAWSDNSSuffix` (76), `partitionDNSSuffixByRegionPrefix` (77–82), endpoint `sts.<region>.<suffix>` (92), API `Version 2011-06-15` + clamp 900–43200 (147, 158).
`// MARK:` `67 (Constants), 85 (Endpoint Helpers), 105 (Role Assumption Async), 257 (Sync), 375 (XML Parsing), 433 (Query String Building), 446 (SigV4 Helper), 499 (STS Response Parser), 530 (XMLParserDelegate)`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 44–103 | | `errorDescription` + konstanta + `endpointHost(for:)` / `endpointDNSSuffix(for:)` | Konstanta + endpoint |
| 108–255 | | `assumeRole(...)` | Request AssumeRole async bertanda SigV4 |
| 275–347 | | `assumeRole(_:...error:)` | Wrapper sync ObjC via background task |
| 354–431 | | `assumeRoleWithMFA(...)` / `parseAssumeRoleResponse` / `parseISO8601Date` / `parseErrorFromXML` / `extractValue` | MFA + parsing |
| 435–496 | | `buildQueryString` / `deriveSigningKey` / `hmacSHA256` / `hmacSHA256Hex` / `sha256Hex` / `hexEncode` / `urlEncode` | SigV4 |
| 503–576 | | `STSResponseParser` (init/parse/parser delegate) | Parser XML |

**Temuan**: SigV4 penuh (canonical request 198, scope 201, string-to-sign 203, signing key 206–212, header `Authorization` 215). Signed headers memuat `x-amz-security-token` bila ada session token (188–196, 226–228). Nama sesi default `SequelAce-<epoch>`; durasi clamp 900–43200 (142–147). MFA param hanya bila serial + code (135–139, 164–168). Wrapper sync pakai `DispatchQueue.global` + semaphore timeout `requestTimeout+5` (296–326). Error XML di-log tapi toleran (572–576). ⚠️ `extractValue` naive first-match string search — bisa salah tangani tag nested/duplikat (**UNKNOWN** robustness) (421–431). **UNKNOWN**: tanpa koreksi clock-skew/retry 5xx (240–249).

---

## 6. `RDSIAMAuthentication.swift` (366)

Generate token auth DB RDS IAM via SigV4 presigned `connect` GET URL + deteksi region dari hostname.

Tipe: `RDSIAMAuthenticationError` (35–50), `RDSIAMAuthentication` (53–330), extension (334–366).
Konstanta: `algorithm`/`service="rds-db"`/`awsRequest`/`connectAction`/`tokenExpirationSeconds = 900` (57–61), `regionPattern` (69)/`regionRegex` (70), konstanta SHA-256 payload kosong (189), port default 3306 (119).
`// MARK:` `55 (Constants), 65 (AWS Region Patterns), 72 (Token Generation), 130 (Token Building), 232 (SigV4 Helper), 285 (Region Detection), 332`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 40–70 | | `errorDescription` + konstanta + regex | Konstanta |
| 83 | 128 | `generateAuthToken(forHost:port:username:region:credentials:)` | Validasi input, resolve region, bangun token |
| 132 | 230 | `buildPresignedToken(host:port:username:region:credentials:)` | Bangun token presigned |
| 234–283 | | `deriveSigningKey` / `hmacSHA256` / `hmacSHA256Hex` / `sha256Hex` / `hexEncode` / `urlEncode` | SigV4 |
| 288–365 | | `regionFromHostname` / `isValidAWSRegion` / `isRDSHostname` / `tokenLifetimeSeconds` / `generateAuthTokenObjC` | Region + ObjC |

**Temuan**: hanya `host` yang ditandatangani; canonical URI `/`, payload hash = konstanta SHA-256 string kosong (154–156, 185–189). Query param alfabetis, `X-Amz-Security-Token` disisip sebelum `X-Amz-SignedHeaders` (166–180); token akhir dirakit **ulang manual** di 229 — duplikasi berisiko drift dari canonical query. Region: eksplisit → deteksi hostname → throw (103–116). **UNKNOWN**: token tak di-cache/track expiry di file ini.

---

## 7. `SAAWSLoginRefresh.swift` (525)

Bangun request refresh DPoP-signed `CreateOAuth2Token`, model sesi cache `aws login`, parse respons token, dan atomic cache replacement.

Tipe: `SAAWSLoginRefreshError` (33–66), `SAAWSSignInEndpoint` (71–108), `SAAWSDPoPProof` (113–168), `SAAWSLoginSession` (173–308), `SAAWSLoginTokenResponse` (313–368), `SAAWSLoginRefreshRequest` (373–431), `SAAtomicFileReplacement` (436–525).
Konstanta: `refreshWindow = 300` (176); `partitionDomains` (73–81) + default (83); token URL `https://<region>.<domain>/v1/token` (95); regex region (87); `timeout = 15` + deadline `timeout+5` (376, 398); grant `refresh_token` (382).
`// MARK:` `68 (Endpoint), 110 (DPoP Proof), 170 (Cached Session), 310 (Token Response), 370 (Request), 433 (Atomic File Replacement)`.

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 44–107 | | `errorDescription` / `partitionDomains` / `defaultDomain` / `isValidRegion` / `tokenURL(forRegion:)` / `region(fromIssuer:)` | Endpoint |
| 116–167 | | `make(privateKeyPEM:url:issuedAt:id:)` / `base64URLEncoded` / `base64URLDecoded` / `jsonData` | DPoP proof |
| 176–307 | | `refreshWindow` / `root` / `init(jsonData:)` / `accessToken` / `expiration` / `refreshToken` / `clientId` / `dpopKey` / `issuingRegion` / `remainingLifetime` / `needsRefresh` / `credentials` / `applying` / `jsonData()` / `parseTimestamp` / `formatTimestamp` / `issuer(ofJWT:)` / `nonEmptyString` | Sesi cache |
| 314–367 | | `SAAWSLoginTokenResponse` fields + `parse(data:statusCode:)` / `error(forStatusCode:body:)` | Parsing respons |
| 376–430 | | `timeout` / `make(url:clientId:refreshToken:dpopKey:issuedAt:)` / `send(_:configuration:deadline:)` | Request |
| 438–523 | | `SAAtomicFileReplacement` (init/deinit/commit/discard/canCreateFiles/temporaryPath) | Atomic replacement |

**Temuan**: DPoP proof JWT ES256 compact: header `typ=dpop+jwt` + JWK `P-256` (x/y raw split 32/32); claims `jti` (UUID lowercase), `htm=POST`, `htu=url`, `iat` (125–140), sign atas `base64url(header).base64url(claims)` (142–145). **UNKNOWN**: tanpa nonce DPoP / claim `ath`/thumbprint (135–140). Parse sesi wajib `accessToken.accessKeyId`/`secretAccessKey` (182–187). `issuingRegion` coba `idToken` lalu `identityToken` (217–227, 294–302). Respons terima `tokenOutput` wrapper atau root (328–334). Error mapping `TOKEN_EXPIRED`/`USER_CREDENTIALS_CHANGED`/`INSUFFICIENT_PERMISSIONS`, lalu 429/5xx → `requestFailed` (348–367). `SAAtomicFileReplacement` buka temp `O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC` 0600 (446), loop write tangani `EINTR` (466–474), `fsync` file (478) + direktori (493–497), `rename` (487). **UNKNOWN**: apakah `applying` benar mempertahankan `dpopKey`/`clientId` — hanya update `accessToken`+`refreshToken` (255–266).

## Ringkasan lintas-file
- Semua error enum punya `errorDescription`; bridge ObjC map rawValue ke `NSError` dengan domain per-file (`AWSCredentialsErrorDomain`, dst).
- Penanganan region/DNS partition **terduplikasi**: STS longest-prefix map (77–103), SSO China-only (156–160), Sign-In partition list (73–96); RDS pakai regex shape saja (69–70).
- Helper SigV4 (`hmacSHA256`, `hmacSHA256Hex`, `sha256Hex`, `hexEncode`, `urlEncode`, `deriveSigningKey`) **copy-paste** antara `AWSSTSClient` (457–496) dan `RDSIAMAuthentication` (243–283).
- Pola baca file security-scoped bookmark-else-direct identik di `AWSCredentials` (336–350), `AWSLoginCredentialsProvider` (244–256), `AWSSSOClient` (316–328).
