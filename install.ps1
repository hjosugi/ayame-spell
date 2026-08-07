[CmdletBinding()]
param(
    [string]$Version = $(if ($env:AYAME_SPELL_VERSION) { $env:AYAME_SPELL_VERSION } else { "latest" }),
    [string]$InstallDir = $(if ($env:AYAME_SPELL_INSTALL_DIR) { $env:AYAME_SPELL_INSTALL_DIR } else { Join-Path $HOME ".local\bin" })
)

$ErrorActionPreference = "Stop"
$Repository = "ayame-editor/ayame-spell"

if ($Version -eq "latest") {
    $Release = Invoke-RestMethod `
        -Headers @{ "User-Agent" = "ayame-spell-installer" } `
        -Uri "https://api.github.com/repos/$Repository/releases/latest"
    $Tag = $Release.tag_name
} elseif ($Version.StartsWith("v")) {
    $Tag = $Version
} else {
    $Tag = "v$Version"
}

$Architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($Architecture -ne [System.Runtime.InteropServices.Architecture]::X64) {
    throw "Unsupported Windows architecture: $Architecture (release binaries currently support x64)"
}

$Target = "x86_64-pc-windows-msvc"
$Archive = "ayame-spell-$Tag-$Target.zip"
$BaseUrl = "https://github.com/$Repository/releases/download/$Tag"
$Temporary = Join-Path ([IO.Path]::GetTempPath()) ("ayame-spell-install-" + [guid]::NewGuid())

try {
    New-Item -ItemType Directory -Path $Temporary | Out-Null
    $ArchivePath = Join-Path $Temporary $Archive
    $SumsPath = Join-Path $Temporary "SHA256SUMS"
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$Archive" -OutFile $ArchivePath
    Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/SHA256SUMS" -OutFile $SumsPath

    $EscapedArchive = [regex]::Escape($Archive)
    $ChecksumLine = Get-Content $SumsPath |
        Where-Object { $_ -match "^\s*([0-9a-fA-F]{64})\s+\*?$EscapedArchive$" } |
        Select-Object -First 1
    if (-not $ChecksumLine) {
        throw "Release checksum is missing for $Archive"
    }
    $Expected = ([regex]::Match($ChecksumLine, "[0-9a-fA-F]{64}")).Value.ToLowerInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 $ArchivePath).Hash.ToLowerInvariant()
    if ($Actual -ne $Expected) {
        throw "Checksum mismatch for $Archive"
    }

    Expand-Archive -Path $ArchivePath -DestinationPath $Temporary
    $Source = Join-Path $Temporary "ayame-spell-$Tag-$Target\ayame-spell.exe"
    if (-not (Test-Path $Source -PathType Leaf)) {
        throw "The release archive does not contain ayame-spell.exe"
    }
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Force $Source (Join-Path $InstallDir "ayame-spell.exe")
    Write-Host "Installed ayame-spell $($Tag.TrimStart('v')) to $InstallDir\ayame-spell.exe"
    if (($env:PATH -split [IO.Path]::PathSeparator) -notcontains $InstallDir) {
        Write-Host "Add $InstallDir to PATH to run ayame-spell."
    }
} finally {
    if (Test-Path $Temporary) {
        Remove-Item -Recurse -Force $Temporary
    }
}
