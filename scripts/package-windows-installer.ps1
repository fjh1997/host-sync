param(
  [string]$Version = "",
  [string]$Target = "",
  [string]$BinariesDir = "",
  [string]$OutDirSuffix = "",
  [string]$OutputName = ""
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
  if ($PSScriptRoot) {
    return (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
  }
  return (Resolve-Path ".").Path
}

function Read-VersionFromCargoToml {
  $cargoToml = Get-Content (Join-Path $repoRoot "Cargo.toml") -Raw
  if ($cargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read workspace version from Cargo.toml"
  }
  return $Matches[1]
}

function ToTomlPath([string]$Path) {
  return ($Path -replace '\\', '/')
}

$repoRoot = Resolve-RepoRoot
Set-Location $repoRoot

if (-not $Version) {
  $Version = Read-VersionFromCargoToml
}

$Version = $Version.TrimStart("v")

if (-not $Target) {
  throw "Target triple is required"
}

if (-not $OutDirSuffix) {
  $OutDirSuffix = $Target
}

if (-not $BinariesDir) {
  $BinariesDir = "target/$Target/release"
}

$outDir = "dist/$OutDirSuffix"
$configPath = Join-Path $repoRoot "packager.generated.toml"
$exePath = Join-Path $repoRoot "$BinariesDir/hostsync.exe"

if (-not (Test-Path -LiteralPath $exePath)) {
  throw "Expected built executable not found: $exePath"
}

$config = Get-Content (Join-Path $repoRoot "packager.toml") -Raw
$config = $config -replace 'version = ".*"', "version = `"$Version`""
$tomlOutDir = ToTomlPath $outDir
$tomlBinariesDir = ToTomlPath $BinariesDir
$config = $config -replace 'out-dir = "\./dist"', "out-dir = `"$tomlOutDir`"`ntarget-triple = `"$Target`""
$config = $config -replace 'binaries-dir = "\./target/release"', "binaries-dir = `"$tomlBinariesDir`""
[System.IO.File]::WriteAllText($configPath, $config, [System.Text.UTF8Encoding]::new($false))

try {
  cargo packager -f nsis --config $configPath --verbose
  if ($LASTEXITCODE -ne 0) {
    throw "cargo packager failed with exit code $LASTEXITCODE"
  }

  $installer = Get-ChildItem -LiteralPath (Join-Path $repoRoot $outDir) -Filter "hostsync_${Version}_*-setup.exe" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if (-not $installer) {
    throw "Failed to locate generated installer in $outDir"
  }

  $destinationName = if ($OutputName) { $OutputName } else { $installer.Name }
  $destinationPath = Join-Path $installer.DirectoryName $destinationName
  if ($destinationPath -ne $installer.FullName) {
    if (Test-Path $destinationPath) {
      Remove-Item -LiteralPath $destinationPath -Force
    }
    Move-Item -LiteralPath $installer.FullName -Destination $destinationPath
  }

  if ($env:GITHUB_OUTPUT) {
    Add-Content -LiteralPath $env:GITHUB_OUTPUT -Value "installer_path=$destinationPath"
  }

  Write-Host "Created installer: $destinationPath"
}
finally {
  if (Test-Path $configPath) {
    Remove-Item -LiteralPath $configPath -Force
  }
}
