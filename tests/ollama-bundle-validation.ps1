param(
    [string]$PayloadRoot = (Join-Path $PSScriptRoot '..\installer\payload')
)

$ErrorActionPreference = 'Stop'
$failures = [System.Collections.Generic.List[string]]::new()

function Add-Failure([string]$Message) {
    $script:failures.Add($Message)
}

$payload = [IO.Path]::GetFullPath($PayloadRoot)
$runtime = Join-Path $payload 'ollama\OllamaSetup.exe'
$hashFile = Join-Path $payload 'SHA256SUMS.txt'
$ollamaLicense = Join-Path $payload 'LICENSES\OLLAMA-MIT.txt'
$modelLicense = Join-Path $payload 'LICENSES\DEEPSEEK-CODER.txt'
$manifestRoot = Join-Path $payload 'models\manifests'

foreach ($required in @($runtime, $hashFile, $ollamaLicense, $modelLicense, $manifestRoot)) {
    if (-not (Test-Path -LiteralPath $required)) {
        Add-Failure "Missing required bundle item: $required"
    }
}

if ($failures.Count -eq 0) {
    $manifest = Get-ChildItem -LiteralPath $manifestRoot -Recurse -File |
        Where-Object { $_.FullName -match 'deepseek-coder[\\/]1\.3b$' } |
        Select-Object -First 1
    if (-not $manifest) {
        Add-Failure 'Missing deepseek-coder:1.3b manifest in bundled model store.'
    } else {
        try {
            $model = Get-Content -Raw -Encoding UTF8 -LiteralPath $manifest.FullName | ConvertFrom-Json
            $digests = @($model.config.digest) + @($model.layers | ForEach-Object { $_.digest }) |
                Where-Object { $_ -is [string] -and $_.StartsWith('sha256:') } |
                Select-Object -Unique
            if ($digests.Count -eq 0) {
                Add-Failure 'The DeepSeek model manifest has no SHA-256 blob references.'
            }
            foreach ($digest in $digests) {
                $blob = Join-Path $payload ('models\blobs\' + ($digest -replace ':', '-'))
                if (-not (Test-Path -LiteralPath $blob)) {
                    Add-Failure "Manifest references a missing blob: $digest"
                }
            }
        } catch {
            Add-Failure "Could not parse model manifest: $($_.Exception.Message)"
        }
    }
}

if ($failures.Count -eq 0) {
    $expectedHashes = @{}
    foreach ($line in Get-Content -Encoding UTF8 -LiteralPath $hashFile) {
        if ($line -match '^([A-Fa-f0-9]{64}) \*?(.+)$') {
            $expectedHashes[$matches[2].Replace('/', '\')] = $matches[1].ToUpperInvariant()
        }
    }
    if ($expectedHashes.Count -eq 0) {
        Add-Failure 'SHA256SUMS.txt does not contain any SHA-256 entries.'
    }
    foreach ($relative in $expectedHashes.Keys) {
        $file = Join-Path $payload $relative
        if (-not (Test-Path -LiteralPath $file)) {
            Add-Failure "Checksum references a missing file: $relative"
            continue
        }
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash.ToUpperInvariant()
        if ($actual -ne $expectedHashes[$relative]) {
            Add-Failure "Checksum mismatch: $relative"
        }
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Output "Ollama bundle validation: PASS ($payload)"
