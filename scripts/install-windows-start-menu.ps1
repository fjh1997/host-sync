param(
  [string]$ExePath = "",
  [string]$InstallDir = "",
  [switch]$PerMachine
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
  if ($PSScriptRoot) {
    return (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
  }
  return (Resolve-Path ".").Path
}

function Test-Admin {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = [Security.Principal.WindowsPrincipal]::new($identity)
  return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

$repoRoot = Resolve-RepoRoot

if (-not $ExePath) {
  $ExePath = Join-Path $repoRoot "target\release\hostsync.exe"
}
$ExePath = (Resolve-Path $ExePath).Path

if (-not $InstallDir) {
  if ($PerMachine) {
    $InstallDir = Join-Path $env:ProgramFiles "HostSync"
  } else {
    $InstallDir = Join-Path $env:LOCALAPPDATA "HostSync"
  }
}

if ($PerMachine -and -not (Test-Admin)) {
  throw "Per-machine install needs an elevated PowerShell session. Re-run as Administrator or omit -PerMachine."
}

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

$targetExe = Join-Path $InstallDir "hostsync.exe"
Copy-Item -LiteralPath $ExePath -Destination $targetExe -Force

$programsRoot = if ($PerMachine) {
  [Environment]::GetFolderPath("CommonPrograms")
} else {
  [Environment]::GetFolderPath("Programs")
}

$shortcutDir = Join-Path $programsRoot "HostSync"
New-Item -ItemType Directory -Path $shortcutDir -Force | Out-Null

$shortcutPath = Join-Path $shortcutDir "HostSync.lnk"
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $targetExe
$shortcut.WorkingDirectory = $InstallDir
$shortcut.IconLocation = "$targetExe,0"
$shortcut.Description = "HostSync"
$shortcut.Save()

if (Get-Command ie4uinit.exe -ErrorAction SilentlyContinue) {
  Start-Process ie4uinit.exe -ArgumentList "-show" -WindowStyle Hidden -Wait -ErrorAction SilentlyContinue
}

Write-Host "Installed HostSync executable: $targetExe"
Write-Host "Created Start menu shortcut: $shortcutPath"
