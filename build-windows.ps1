# Build the Rust port natively on Windows.
#
# Requires PowerShell 5.1+ (preinstalled on Windows 10/11), a Rust toolchain,
# and libmysqlclient. The driver is libmysqlclient itself (see README.md), so
# the port is linked against the same C client library the macOS app uses.
#
# The script supplies libmysqlclient one of two ways and then runs the same
# checks the CI workflow runs:
#
#   1. vcpkg (recommended). Install once:
#          vcpkg install libmysql:x64-windows
#      VCPKG_ROOT is picked up from the environment (or C:\vcpkg).
#
#   2. An already installed MySQL Connector/C (or Community Server):
#          $env:MYSQLCLIENT_LIB_DIR     = 'C:\Program Files\MySQL\<v>\lib'
#          $env:MYSQLCLIENT_INCLUDE_DIR = 'C:\Program Files\MySQL\<v>\include'
#          $env:MYSQLCLIENT_VERSION     = '8.4.11'
#
# For parity with the macOS app, use MySQL 8.4.x. Any version MySQL 5.7-9.5 or
# MariaDB 3.1/3.3/3.4 compiles; a different one changes cipher defaults, auth
# plugins, and error text.
#
# The x64-windows triplet gives the classical MySQL layout: libmysql.dll plus
# the libmysql.lib import library, which is exactly what MYSQLCLIENT_LIBNAME
# defaults to in mysqlclient-sys. A statically linked client complicates the
# link (it needs matching OpenSSL/zlib/zstd static imports), so the script
# keeps the dynamic layout and copies the needed DLLs next to the artifact.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$triplet = 'x64-windows'
$artifactName = 'SequelAce-Windows-x64.exe'
# Source `install-exports.patch` in vcpkg's libmysql port: the installed export
# name is uniform (`MYSQLCLIENT_LIBNAME=libmysql`), so set it explicitly to be
# safe in both the vcpkg-driven path in mysqlclient-sys and the fallback path.
$clientLibName = 'libmysql'
# Set in Resolve-ClientLibrary so New-Artifact can find the runtime DLLs.
$script:clientLibRoot = $null
# Sibling dirs that carry libmysql.dll's runtime dependencies (libcrypto-3-x64.dll,
# libssl-3-x64.dll — inside a MySQL zip they live in bin\, not lib\).
$script:clientDepRoots = @()

# On a Windows ARM64 host (UTM/Parallels VMs) the build still targets x64 —
# MySQL ships its Windows client library for x64 only. The script detects an
# ARM64 host automatically and cross-builds with the native ARM64 compiler
# (setup A below). On a real x64 machine nothing is forced.
#
#   A. Native ARM64 compiler + x64 target (faster, automatic on ARM64 hosts):
#        .\build-windows.ps1
#
#   B. x64 toolchain under Prism emulation (identical to a real x64 machine,
#      slower builds):
#        rustup toolchain install stable-x86_64-pc-windows-msvc --force-non-host
#        rustup default stable-x86_64-pc-windows-msvc
#        $env:CARGO_BUILD_TARGET = ''   # explicitly unset to skip the target
#        .\build-windows.ps1
#
# With setup A cargo writes artifacts into target\<CARGO_BUILD_TARGET>\release\
# instead of target\release\, so every consumer below goes through
# Resolve-ReleaseDir.

function Resolve-ReleaseDir {
    if (Test-Path 'Env:CARGO_BUILD_TARGET') {
        return Join-Path $projectDir "target\$($env:CARGO_BUILD_TARGET)\release"
    }
    return Join-Path $projectDir 'target\release'
}

function Write-Section {
    param([string] $Text)
    Write-Host ''
    Write-Host "== $Text =="
}

# ARM64 host detection (UTM VM on Apple Silicon): force the x64 target so
# cargo cross-compiles with the native ARM64 rustc. An x64 host is untouched.
if (-not (Test-Path 'Env:CARGO_BUILD_TARGET') -and $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    $env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
    Write-Host 'ARM64 host detected: cross-building for x86_64-pc-windows-msvc.'
}

function Resolve-RustToolchain {
    Write-Section 'Rust toolchain'

    Write-Host "Host rustup default: $(rustup default 2>$null)"
    if (Test-Path 'Env:CARGO_BUILD_TARGET') {
        Write-Host "Cross target: $env:CARGO_BUILD_TARGET"
        # Idempotent; fails clearly if the std for the target is missing.
        rustup target add $env:CARGO_BUILD_TARGET
        if ($LASTEXITCODE -ne 0) {
            throw "Could not install the $($env:CARGO_BUILD_TARGET) target (rustup target add)."
        }
    } else {
        # Idempotent; also provisions clippy and rustfmt on a fresh machine.
        rustup toolchain install stable --profile minimal --component clippy,rustfmt
        if ($LASTEXITCODE -ne 0) {
            throw 'Failed to install the stable Rust toolchain.'
        }
    }
}

function Resolve-VcpkgRoot {
    if (Test-Path 'Env:VCPKG_ROOT') {
        return $env:VCPKG_ROOT
    }
    if (Test-Path 'Env:VCPKG_INSTALLATION_ROOT') {
        return $env:VCPKG_INSTALLATION_ROOT
    }
    if (Test-Path 'C:\vcpkg') {
        return 'C:\vcpkg'
    }
    return $null
}

# vcpkg's mysql port installs the header under include\mysql, with the version
# recorded in mysql_version.h.
function Read-ClientVersion {
    param([string] $PackageHeader)

    $header = Get-Content $PackageHeader -Raw
    if ($header -match 'LIBMYSQL_VERSION\s+"(\d+\.\d+\.\d+)"') {
        return $Matches[1]
    }
    if ($header -match 'MYSQL_VERSION_ID\s+"?(\d+)"?') {
        # 80046 -> 8.0.46
        $versionId = [int] $Matches[1]
        $major = [math]::Floor($versionId / 10000)
        $minor = [math]::Floor(($versionId % 10000) / 100)
        $patch = $versionId % 100
        return "$major.$minor.$patch"
    }
    throw "Could not read the installed libmysqlclient version from $PackageHeader"
}

function Find-ClientRoot {
    # Auto-discover a MySQL client install when the MYSQLCLIENT_* env vars are
    # not set. Checked in order; the first layout with both lib+include wins
    # (an extracted mysql-8.4.x-winx64.zip and an MSI server install both fit).
    $patterns = @(
        'C:\mysql*\*\lib\libmysql.lib',      # C:\mysql84\mysql-8.4.x-winx64\lib
        'C:\mysql*\lib\libmysql.lib',        # flattened extraction
        'C:\Program Files\MySQL\MySQL Server*\lib\libmysql.lib',
        'C:\Program Files (x86)\MySQL\MySQL Connector C*\lib\vs*\libmysql.lib'
    )
    foreach ($pattern in $patterns) {
        $hits = @(Get-ChildItem -Path $pattern -ErrorAction SilentlyContinue |
            Where-Object { -not ($_.PSIsContainer) })
        foreach ($hit in $hits) {
            $root = $hit.Directory.Parent.FullName
            if (Test-Path (Join-Path $root 'include\mysql_version.h')) {
                return $root
            }
        }
    }
    return $null
}

function Resolve-ClientLibrary {
    Write-Section 'libmysqlclient'

    # Manual override for MYSQLCLIENT_LIB_DIR / MYSQLCLIENT_VERSION. Either
    # works on its own, so require both to keep the meaning unambiguous.
    if ((Test-Path 'Env:MYSQLCLIENT_LIB_DIR') -and (Test-Path 'Env:MYSQLCLIENT_VERSION')) {
        $env:MYSQLCLIENT_LIBNAME = $clientLibName
        $script:clientLibRoot = $env:MYSQLCLIENT_LIB_DIR
        $env:PATH = "$script:clientLibRoot;$env:PATH"
        # libmysql.dll's deps (libcrypto/libssl) sit in the sibling bin\ dir of
        # a MySQL zip/server install; register it when it exists.
        $binDir = Join-Path (Split-Path $env:MYSQLCLIENT_LIB_DIR -Parent) 'bin'
        if (Test-Path $binDir) {
            $script:clientDepRoots += $binDir
            $env:PATH = "$binDir;$env:PATH"
        }
        Write-Host "Using MYSQLCLIENT_LIB_DIR: $env:MYSQLCLIENT_LIB_DIR"
        Write-Host "libmysqlclient version: $env:MYSQLCLIENT_VERSION"
        return
    }

    # No env vars and no vcpkg? Discover a local MySQL install automatically —
    # e.g. an extracted mysql-8.4.x-winx64.zip or an MSI-installed server dir.
    if ($null -eq (Resolve-VcpkgRoot)) {
        $discovered = Find-ClientRoot
        if ($null -ne $discovered) {
            $env:MYSQLCLIENT_LIB_DIR = Join-Path $discovered 'lib'
            $env:MYSQLCLIENT_INCLUDE_DIR = Join-Path $discovered 'include'
            $versionHeader = Join-Path $env:MYSQLCLIENT_INCLUDE_DIR 'mysql_version.h'
            $env:MYSQLCLIENT_VERSION = Read-ClientVersion $versionHeader
            $env:MYSQLCLIENT_LIBNAME = $clientLibName
            $script:clientLibRoot = $env:MYSQLCLIENT_LIB_DIR
            $env:PATH = "$script:clientLibRoot;$env:PATH"
            # Sibling bin\ dir carries libcrypto/libssl dependencies.
            $binDir = Join-Path $discovered 'bin'
            if (Test-Path $binDir) {
                $script:clientDepRoots += $binDir
                $env:PATH = "$binDir;$env:PATH"
            }
            Write-Host "Discovered MySQL client: $discovered"
            Write-Host "libmysqlclient version: $env:MYSQLCLIENT_VERSION"
            return
        }
    }

    $vcpkgRoot = Resolve-VcpkgRoot
    if ($null -eq $vcpkgRoot) {
        Write-Host ''
        Write-Host 'No libmysqlclient is available to link against.'
        Write-Host ''
        Write-Host 'Install it through vcpkg (recommended):'
        Write-Host "    vcpkg install libmysql:$triplet"
        Write-Host '    $env:VCPKG_ROOT = "<vcpkg installation directory>"'
        Write-Host ''
        Write-Host 'Or point at an already installed MySQL client:'
        Write-Host '    $env:MYSQLCLIENT_VERSION     = "8.4.11"'
        Write-Host '    $env:MYSQLCLIENT_LIB_DIR     = "<path to libmysqlclient.lib>"'
        Write-Host '    $env:MYSQLCLIENT_INCLUDE_DIR = "<path to mysql.h>"'
        throw 'No libmysqlclient is available to link against.'
    }

    $env:VCPKG_ROOT = $vcpkgRoot
    $env:VCPKG_DEFAULT_TRIPLET = $triplet

    $installedDir = Join-Path $vcpkgRoot "installed\$triplet"
    $packageHeader = Join-Path $installedDir 'include\mysql\mysql_version.h'
    # Also point mysqlclient-sys's own fallback at the same install, so the
    # build still works when its vcpkg auto-detection cannot see the port.
    $script:clientLibRoot = Join-Path $installedDir 'lib'

    if (-not (Test-Path $packageHeader)) {
        Write-Host "Installing libmysqlclient via vcpkg ($triplet). This can take a while."
        & (Join-Path $vcpkgRoot 'vcpkg.exe') install "libmysql:$triplet"
        if ($LASTEXITCODE -ne 0) {
            throw "vcpkg install libmysql:$triplet failed in $vcpkgRoot"
        }
    }

    if (-not (Test-Path 'Env:MYSQLCLIENT_VERSION')) {
        $env:MYSQLCLIENT_VERSION = Read-ClientVersion $packageHeader
    }
    $env:MYSQLCLIENT_LIBNAME = $clientLibName
    # Make the runtime DLLs visible to cargo test (dynamic linking).
    $env:PATH = "$script:clientLibRoot;$env:PATH"
    $vcpkgBin = Join-Path $installedDir 'bin'
    if (Test-Path $vcpkgBin) {
        $script:clientDepRoots += $vcpkgBin
        $env:PATH = "$vcpkgBin;$env:PATH"
    }
    Write-Host "vcpkg root: $vcpkgRoot ($triplet)"
    Write-Host "libmysqlclient version: $env:MYSQLCLIENT_VERSION"
}

function Invoke-Checks {
    Write-Section 'Check formatting, tests, lint, and build'

    Push-Location $projectDir
    try {
        cargo +stable fmt --all --check
        if ($LASTEXITCODE -ne 0) { throw 'cargo fmt --check found formatting problems.' }

        cargo +stable test --locked
        if ($LASTEXITCODE -ne 0) { throw 'cargo test failed.' }

        cargo +stable clippy --all-targets --locked -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw 'cargo clippy reported warnings.' }

        cargo +stable build --release --locked
        if ($LASTEXITCODE -ne 0) { throw 'cargo build --release failed.' }
    } finally {
        Pop-Location
    }
}

function New-Artifact {
    Write-Section 'Artifact'

    $artifactDir = Join-Path $projectDir 'dist'
    $releaseDir = Resolve-ReleaseDir
    $builtExe = Join-Path $releaseDir 'sequel-ace-windows.exe'
    $builtAskPass = Join-Path $releaseDir 'sequel-ace-askpass.exe'
    if (-not (Test-Path $builtExe)) {
        throw "Expected build output at $builtExe"
    }
    if (-not (Test-Path $builtAskPass)) {
        throw "Expected SSH_ASKPASS helper at $builtAskPass (missing [[bin]] target?)"
    }
    New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
    Copy-Item $builtExe (Join-Path $artifactDir $artifactName) -Force
    # SSH_ASKPASS helper must ship next to the main exe (src/bin/sequel-ace-askpass.rs).
    Copy-Item $builtAskPass (Join-Path $artifactDir 'sequel-ace-askpass.exe') -Force

    # The x64-windows layout links libmysqlclient dynamically, so the artifact
    # needs the client's runtime DLLs — libmysql.dll from lib\ plus its deps
    # (libcrypto/libssl) from the sibling bin\ dir — next to the .exe.
    $dllSources = @()
    if ($null -ne $script:clientLibRoot) {
        $dllSources += $script:clientLibRoot
    }
    if ($script:clientDepRoots.Count -gt 0) {
        $dllSources += $script:clientDepRoots
    }

    foreach ($dllSource in $dllSources) {
        Write-Host "Looking for runtime DLLs in $dllSource"
        $runtimeDlls = @(Get-ChildItem -Path (Join-Path $dllSource '*.dll') -ErrorAction SilentlyContinue)
        if ($runtimeDlls.Count -eq 0) {
            Write-Host '  none found'
            continue
        }
        Copy-Item $runtimeDlls $artifactDir -Force
        Write-Host ("  copied {0}" -f $runtimeDlls.Count)
    }

    Write-Host ''
    Write-Host ("Built artifact in {0}:" -f $artifactDir)
    foreach ($item in @((Join-Path $artifactDir $artifactName),
                        (Join-Path $artifactDir 'sequel-ace-askpass.exe'))) {
        $size = [math]::Round((Get-Item $item).Length / 1MB, 1)
        Write-Host ("  {0}  ({1} MB)" -f (Split-Path -Leaf $item), $size)
    }
    Write-Host 'Keep the libmysqlclient DLLs and both .exe files in the same folder.'
}


Resolve-RustToolchain
Resolve-ClientLibrary
Invoke-Checks
New-Artifact
