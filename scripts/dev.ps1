# Lance Runa en mode développement avec watch + relance automatique.
# Usage:
#   .\scripts\dev.ps1              # tauri dev + feature llama (défaut)
#   .\scripts\dev.ps1 -NoLlama     # sans feature llama (stub IA)
#   .\scripts\dev.ps1 -Frontend    # Vite seul (HMR)
#   .\scripts\dev.ps1 -Once        # un seul lancement, sans watch externe

[CmdletBinding()]
param(
    [switch]$Frontend,
    [switch]$Once,
    [switch]$NoLlama,
    [double]$DebounceSeconds = 1.5,
    [int]$MaxFailStreak = 3,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Write-RunaLog {
    param([string]$Message)
    Write-Host "[runa-dev] $Message" -ForegroundColor Cyan
}

function Write-RunaErr {
    param([string]$Message)
    Write-Host "[runa-dev] $Message" -ForegroundColor Red
}

if ($Help) {
    @"
Usage: .\scripts\dev.ps1 [options]

  (défaut)     npm run tauri dev --features llama + FileSystemWatcher (relance)
  -NoLlama     Désactive la feature llama (stub IA ; pas de CMake / LLVM)
  -Frontend    Vite seul (HMR frontend)
  -Once        Lance tauri/vite une fois, sans watch externe
  -DebounceSeconds <n>  Délai avant relance (défaut: 1.5)
  -MaxFailStreak <n>    Après N échecs rapides, attendre un changement source (défaut: 3)
  -Help        Aide
"@ | Write-Host
    exit 0
}

function Get-TauriDevArgs {
    if ($NoLlama) {
        return "run tauri -- dev"
    }
    return "run tauri -- dev --features llama"
}

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Desktop = Join-Path $Root "apps\desktop"
$ExePath = Join-Path $Root "target\debug\runa-desktop.exe"
$WatchDirs = @(
    (Join-Path $Root "crates"),
    (Join-Path $Root "apps\desktop\src-tauri")
)
$VitePort = 1420

# État partagé avec les handlers FileSystemWatcher (autre runspace)
$State = [hashtable]::Synchronized(@{
        RestartRequested = $false
        LastRestart      = Get-Date
        FailStreak       = 0
        PausedForFails   = $false
    })

function Ensure-Deps {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-RunaErr "cargo introuvable — installez Rust (https://rustup.rs)"
        exit 1
    }
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-RunaErr "npm introuvable — installez Node.js"
        exit 1
    }
    $nodeModules = Join-Path $Desktop "node_modules"
    if (-not (Test-Path $nodeModules)) {
        Write-RunaLog "npm install (apps/desktop)…"
        Push-Location $Desktop
        try { npm install } finally { Pop-Location }
    }
}

function Get-NpmCommand {
    $cmd = Get-Command npm.cmd -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return (Get-Command npm).Source
}

function Stop-PortListeners {
    param([int]$Port)
    try {
        $conns = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        foreach ($c in $conns) {
            $procId = $c.OwningProcess
            if ($procId -and $procId -ne 0) {
                Write-RunaLog "libération port $Port (PID $procId)"
                Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
            }
        }
    } catch {
        # Get-NetTCPConnection peut échouer selon droits / OS
    }
}

function Test-ExeWritable {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return $true }
    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::ReadWrite,
            [System.IO.FileShare]::None
        )
        $stream.Close()
        return $true
    } catch {
        return $false
    }
}

function Wait-RunaExeUnlocked {
    param([int]$TimeoutSeconds = 15)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-ExeWritable -Path $ExePath) {
            return $true
        }
        Start-Sleep -Milliseconds 400
    }
    return (Test-ExeWritable -Path $ExePath)
}

function Stop-RunaTree {
    param([System.Diagnostics.Process]$Process)

    # Arbre npm → tauri → vite / cargo / runa-desktop
    Stop-Process -Name "runa-desktop" -Force -ErrorAction SilentlyContinue
    Stop-PortListeners -Port $VitePort

    if ($null -eq $Process) {
        Start-Sleep -Milliseconds 500
        if (-not (Wait-RunaExeUnlocked -TimeoutSeconds 10)) {
            Write-RunaErr "runa-desktop.exe encore verrouillé après kill — fermez l'app manuellement"
        }
        return
    }
    if (-not $Process.HasExited) {
        Write-RunaLog "arrêt du processus $($Process.Id)…"
        try {
            $toKill = New-Object System.Collections.Generic.List[int]
            $toKill.Add($Process.Id) | Out-Null
            $queue = [System.Collections.Generic.Queue[int]]::new()
            $queue.Enqueue($Process.Id)
            while ($queue.Count -gt 0) {
                $parent = $queue.Dequeue()
                Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
                    Where-Object { $_.ParentProcessId -eq $parent } |
                    ForEach-Object {
                        $toKill.Add($_.ProcessId) | Out-Null
                        $queue.Enqueue($_.ProcessId)
                    }
            }
            foreach ($id in ($toKill | Select-Object -Unique)) {
                Stop-Process -Id $id -Force -ErrorAction SilentlyContinue
            }
            $Process.WaitForExit(5000) | Out-Null
        } catch {
            Write-RunaErr "impossible de tuer le processus: $_"
        }
    }

    Start-Sleep -Milliseconds 500
    Stop-Process -Name "runa-desktop" -Force -ErrorAction SilentlyContinue
    Stop-PortListeners -Port $VitePort

    if (-not (Wait-RunaExeUnlocked -TimeoutSeconds 12)) {
        Write-RunaErr "runa-desktop.exe verrouillé — relance annulée tant que le fichier est pris"
    }
}

function Start-TauriDev {
    Stop-RunaTree -Process $null
    if (-not (Wait-RunaExeUnlocked -TimeoutSeconds 8)) {
        Write-RunaErr "impossible de démarrer: $ExePath est verrouillé"
        return $null
    }
    $tauriArgs = Get-TauriDevArgs
    Write-RunaLog "démarrage: npm $tauriArgs"
    if ($NoLlama) {
        Write-RunaLog "feature llama désactivée (stub IA)"
    } else {
        Write-RunaLog "feature llama active (LIBCLANG_PATH / CMake requis)"
    }
    $npm = Get-NpmCommand
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $npm
    $psi.Arguments = $tauriArgs
    $psi.WorkingDirectory = $Desktop
    $psi.UseShellExecute = $false
    return [System.Diagnostics.Process]::Start($psi)
}

function Start-FrontendDev {
    Write-RunaLog "démarrage: npm run dev (Vite)"
    Push-Location $Desktop
    try { npm run dev } finally { Pop-Location }
}

Ensure-Deps

if ($Frontend) {
    Start-FrontendDev
    exit 0
}

if ($Once) {
    $tauriArgs = Get-TauriDevArgs
    Write-RunaLog "tauri dev (sans watch externe)… npm $tauriArgs"
    Stop-RunaTree -Process $null
    Push-Location $Desktop
    try {
        if ($NoLlama) {
            npm run tauri -- dev
        } else {
            npm run tauri -- dev --features llama
        }
    } finally { Pop-Location }
    exit 0
}

Write-RunaLog "mode watch — Ctrl+C pour quitter"
Write-RunaLog "surveillance: crates/, apps/desktop/src-tauri/"
Write-RunaLog "après $MaxFailStreak échecs rapides: pause jusqu'à un changement de source"

$Child = Start-TauriDev
$watchers = New-Object System.Collections.Generic.List[System.IO.FileSystemWatcher]
$subscriberIds = New-Object System.Collections.Generic.List[string]

$onChange = {
    $path = $Event.SourceEventArgs.FullPath
    if ($path -match '\\target\\|\\node_modules\\|\\\.git\\|\\.gguf$|\\gen\\|bindings\.ts') {
        return
    }
    if ($path -notmatch '\.(rs|toml|json)$') {
        return
    }
    # Ignorer les artefacts Tauri / Cargo qui ne sont pas du code source
    if ($path -match '\\capabilities\\|Cargo\.lock$') {
        return
    }
    $Event.MessageData.RestartRequested = $true
    $Event.MessageData.PausedForFails = $false
}

foreach ($dir in $WatchDirs) {
    if (-not (Test-Path $dir)) {
        Write-RunaErr "dossier absent: $dir"
        continue
    }
    $watcher = New-Object System.IO.FileSystemWatcher
    $watcher.Path = $dir
    $watcher.IncludeSubdirectories = $true
    $watcher.NotifyFilter = [IO.NotifyFilters]::FileName -bor
    [IO.NotifyFilters]::DirectoryName -bor
    [IO.NotifyFilters]::LastWrite
    $watcher.Filter = "*.*"
    $watcher.EnableRaisingEvents = $true

    foreach ($evt in @("Changed", "Created", "Deleted", "Renamed")) {
        $sub = Register-ObjectEvent -InputObject $watcher -EventName $evt -Action $onChange -MessageData $State
        $subscriberIds.Add($sub.Name) | Out-Null
    }
    $watchers.Add($watcher) | Out-Null
}

try {
    while ($true) {
        Start-Sleep -Milliseconds 500

        # Sortie inattendue : ne pas boucler indéfiniment (lock exe / compile fail).
        if ($Child -and $Child.HasExited -and -not $State.RestartRequested) {
            if ($State.PausedForFails) {
                continue
            }
            $code = $Child.ExitCode
            $State.FailStreak = [int]$State.FailStreak + 1
            Stop-Process -Name "runa-desktop" -Force -ErrorAction SilentlyContinue

            if ([int]$State.FailStreak -ge $MaxFailStreak) {
                $State.PausedForFails = $true
                $State.RestartRequested = $false
                Write-RunaErr "échec répété (code $code, streak $($State.FailStreak)) — pause."
                Write-RunaErr "Corrigez / libérez runa-desktop.exe, puis modifiez un .rs pour relancer."
                continue
            }

            $backoff = [Math]::Min(2 * [int]$State.FailStreak, 8)
            Write-RunaLog "processus terminé (code $code) — backoff ${backoff}s ($($State.FailStreak)/$MaxFailStreak)…"
            Start-Sleep -Seconds $backoff
            $State.RestartRequested = $true
        }

        if ($State.RestartRequested) {
            $elapsed = ((Get-Date) - [datetime]$State.LastRestart).TotalSeconds
            if ($elapsed -lt $DebounceSeconds) {
                continue
            }
            $State.RestartRequested = $false
            $State.LastRestart = Get-Date
            Start-Sleep -Milliseconds ([int]($DebounceSeconds * 1000))
            $State.RestartRequested = $false
            Write-RunaLog "relance tauri…"
            Stop-RunaTree -Process $Child
            if (-not (Wait-RunaExeUnlocked -TimeoutSeconds 12)) {
                Write-RunaErr "exe verrouillé — pas de relance (modifiez un fichier source pour réessayer)"
                $State.PausedForFails = $true
                $Child = $null
                continue
            }
            $Child = Start-TauriDev
            if ($null -eq $Child) {
                $State.PausedForFails = $true
                continue
            }
            if ($Child -and -not $Child.HasExited) {
                Start-Sleep -Seconds 4
                if ($Child -and -not $Child.HasExited) {
                    $State.FailStreak = 0
                    $State.PausedForFails = $false
                }
            }
        }
    }
} finally {
    Write-RunaLog "nettoyage…"
    foreach ($name in $subscriberIds) {
        Unregister-Event -SourceIdentifier $name -ErrorAction SilentlyContinue
        Remove-Job -Name $name -Force -ErrorAction SilentlyContinue
    }
    foreach ($w in $watchers) {
        $w.EnableRaisingEvents = $false
        $w.Dispose()
    }
    Stop-RunaTree -Process $Child
}
