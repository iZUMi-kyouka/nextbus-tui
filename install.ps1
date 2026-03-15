param(
    [string]$Repo = "iZUMi-kyouka/nextbus-tui",
    [switch]$WhatIf
)

$ErrorActionPreference = "Stop"

function Write-Info {
    param([string]$Message)
    Write-Host "[installer] $Message"
}

function Confirm-Choice {
    param(
        [string]$Prompt,
        [bool]$DefaultYes = $true
    )

    $suffix = if ($DefaultYes) { "[Y/n]" } else { "[y/N]" }
    while ($true) {
        $response = Read-Host "$Prompt $suffix"
        if ([string]::IsNullOrWhiteSpace($response)) {
            return $DefaultYes
        }

        switch -Regex ($response.Trim()) {
            "^(y|yes)$" { return $true }
            "^(n|no)$" { return $false }
            default { Write-Info "Please answer y or n." }
        }
    }
}

function Resolve-Asset {
    param(
        [object]$Release,
        [string[]]$Candidates
    )

    foreach ($name in $Candidates) {
        $hit = $Release.assets | Where-Object { $_.name -eq $name } | Select-Object -First 1
        if ($null -ne $hit) {
            return $hit
        }
    }

    return $null
}

function New-Shortcut {
    param(
        [string]$ShortcutPath,
        [string]$TargetPath,
        [string]$WorkingDirectory
    )

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($ShortcutPath)
    $shortcut.TargetPath = $TargetPath
    $shortcut.WorkingDirectory = $WorkingDirectory
    $shortcut.IconLocation = "$TargetPath,0"
    $shortcut.Save()
}

function Test-IsAdmin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-PathContainsDir {
    param([string]$PathList, [string]$Dir)
    if ([string]::IsNullOrWhiteSpace($PathList)) {
        return $false
    }

    $items = $PathList.Split(';') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
    foreach ($item in $items) {
        if ($item.TrimEnd('\\') -ieq $Dir.TrimEnd('\\')) {
            return $true
        }
    }
    return $false
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
$archTag = switch ($arch) {
    "X64" { "x86_64" }
    "Arm64" { "aarch64" }
    default { throw "Unsupported Windows architecture: $arch" }
}

Write-Info "Detected Windows architecture: $archTag"

$apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
Write-Info "Resolving latest release from $Repo"
$release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "nextbus-tui-installer" }

$candidates = @(
    "nextbus-tui-windows-$archTag.exe",
    "nextbus-tui-windows-$archTag.zip"
)

$asset = Resolve-Asset -Release $release -Candidates $candidates
if ($null -eq $asset) {
    if ($archTag -eq "x86_64") {
        $asset = Resolve-Asset -Release $release -Candidates @(
            "nextbus-tui-windows-x86_64.exe",
            "nextbus-tui-windows-x86_64.zip"
        )
    }
}

if ($null -eq $asset) {
    Write-Host "[installer] ERROR: No matching Windows asset found in latest release." -ForegroundColor Red
    Write-Host "[installer] Available assets:" -ForegroundColor Red
    $release.assets | ForEach-Object { Write-Host "  - $($_.name)" }
    exit 1
}

Write-Info "Selected asset: $($asset.name)"

$installToProgramFiles = Confirm-Choice -Prompt "Install to Program Files?" -DefaultYes $true
if ($installToProgramFiles) {
    if (-not (Test-IsAdmin)) {
        Write-Warning "Program Files install requires an elevated PowerShell session."
        if (Confirm-Choice -Prompt "Use per-user install in LocalAppData instead?" -DefaultYes $true) {
            $installToProgramFiles = $false
        } else {
            throw "Please rerun this installer as Administrator for Program Files installation."
        }
    }
}

if ($installToProgramFiles) {
    $installDir = Join-Path $env:ProgramFiles "nextbus-tui"
} else {
    $installDir = Join-Path $env:LOCALAPPDATA "nextbus-tui"
}

$exePath = Join-Path $installDir "nextbus-tui.exe"

if ($WhatIf) {
    Write-Info "WHATIF: Would download $($asset.browser_download_url)"
    Write-Info "WHATIF: Would install to $exePath"
    Write-Info "WHATIF: Would optionally create desktop/start menu shortcuts"
    exit 0
}

$tempDir = Join-Path $env:TEMP ("nextbus-tui-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

try {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null

    $downloadPath = Join-Path $tempDir $asset.name
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $downloadPath

    if ($asset.name.EndsWith(".zip")) {
        Expand-Archive -Path $downloadPath -DestinationPath $tempDir -Force
        $binary = Get-ChildItem -Path $tempDir -Recurse -File -Filter "nextbus-tui.exe" | Select-Object -First 1
        if ($null -eq $binary) {
            throw "Could not find nextbus-tui.exe inside downloaded zip"
        }
        Copy-Item -Path $binary.FullName -Destination $exePath -Force
    } else {
        Copy-Item -Path $downloadPath -Destination $exePath -Force
    }

    Unblock-File -Path $exePath -ErrorAction SilentlyContinue
}
catch {
    if ($installToProgramFiles) {
        Write-Warning "Program Files install failed. Try running PowerShell as Administrator or choose per-user install."
    }
    throw
}
finally {
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Info "Installed binary to $exePath"

if (Confirm-Choice -Prompt "Also create terminal command alias 'nnbus'?" -DefaultYes $true) {
    $aliasPath = Join-Path $installDir "nnbus.cmd"
    @(
        "@echo off",
        '"%~dp0nextbus-tui.exe" %*'
    ) | Set-Content -Path $aliasPath -Encoding ASCII
    Write-Info "Terminal alias created at $aliasPath"

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not (Test-PathContainsDir -PathList $userPath -Dir $installDir)) {
        if (Confirm-Choice -Prompt "Add install folder to your user PATH so 'nnbus' works in new terminals?" -DefaultYes $true) {
            $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $installDir } else { "$userPath;$installDir" }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Info "Updated user PATH. Open a new terminal session to use 'nnbus'."
        }
    } else {
        Write-Info "Install folder is already in user PATH."
    }
}

if (Confirm-Choice -Prompt "Create Desktop shortcut for double-click launch?" -DefaultYes $true) {
    $desktop = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = Join-Path $desktop "nextbus-tui.lnk"
    New-Shortcut -ShortcutPath $shortcutPath -TargetPath $exePath -WorkingDirectory $installDir
    Write-Info "Desktop shortcut created at $shortcutPath"
}

if (Confirm-Choice -Prompt "Create Start Menu shortcut?" -DefaultYes $true) {
    $startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    $shortcutPath = Join-Path $startMenuDir "nextbus-tui.lnk"
    New-Shortcut -ShortcutPath $shortcutPath -TargetPath $exePath -WorkingDirectory $installDir
    Write-Info "Start Menu shortcut created at $shortcutPath"
}

Write-Info "Install completed."
Write-Info "You can launch from the shortcut or run: $exePath"



