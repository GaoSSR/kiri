$ErrorActionPreference = "Stop"

$Repo = if ($env:KIRI_REPO) { $env:KIRI_REPO } else { "GaoSSR/kiri" }
$Version = if ($env:KIRI_VERSION) { $env:KIRI_VERSION } else { "latest" }
$InstallDir = if ($env:KIRI_INSTALL_DIR) {
    $env:KIRI_INSTALL_DIR
} else {
    Join-Path $HOME ".local\bin"
}

function Fail {
    param([string]$Message)
    Write-Error "kiri: $Message"
    exit 1
}

function Resolve-Target {
    $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        if ($architecture -eq [System.Runtime.InteropServices.Architecture]::X64) {
            return "x86_64-pc-windows-msvc"
        }
        Fail "unsupported Windows architecture: $architecture"
    }

    Fail "PowerShell installer currently supports Windows x64 only."
}

function Release-Url {
    param([string]$Artifact)
    if ($Version -eq "latest") {
        return "https://github.com/$Repo/releases/latest/download/$Artifact"
    }
    return "https://github.com/$Repo/releases/download/$(Release-Tag)/$Artifact"
}

function Release-Tag {
    if ($Version.StartsWith("v")) {
        return $Version
    }
    return "v$Version"
}

$Target = Resolve-Target
$Artifact = "kiri-$Target.zip"
$Url = Release-Url -Artifact $Artifact
$TempDir = New-Item -ItemType Directory -Path (Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString()))

try {
    $ArchivePath = Join-Path $TempDir.FullName $Artifact
    Write-Host "Installing Kiri for $Target"
    Write-Host "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $ArchivePath
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $TempDir.FullName -Force

    $BinaryPath = Join-Path $TempDir.FullName "ports.exe"
    if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
        Fail "release archive did not contain ports.exe"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $Destination = Join-Path $InstallDir "ports.exe"
    Copy-Item -LiteralPath $BinaryPath -Destination $Destination -Force

    Write-Host "Installed ports.exe to $Destination"
    Write-Host "Add $InstallDir to PATH if your shell cannot find ports."
    Write-Host "Next: ports"
} finally {
    Remove-Item -LiteralPath $TempDir.FullName -Recurse -Force -ErrorAction SilentlyContinue
}
