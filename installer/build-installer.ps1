$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'tests\ollama-bundle-validation.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Ollama bundle validation failed.' }

cargo build --release
Copy-Item -LiteralPath 'target\release\commandblock.exe' -Destination 'Commandblock.exe' -Force

$iscc = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source
if (-not $iscc) {
    $candidates = @(
        'C:\Program Files\Inno Setup 6\ISCC.exe',
        'C:\Program Files (x86)\Inno Setup 6\ISCC.exe',
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe')
    )
    $iscc = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}
if (-not (Test-Path -LiteralPath $iscc)) {
    throw 'Inno Setup 6 is required. Install JRSoftware.InnoSetup, then rerun this script.'
}

& $iscc 'installer\Commandblock.iss'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
