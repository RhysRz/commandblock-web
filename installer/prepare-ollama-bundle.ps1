[CmdletBinding()]
param(
    [switch]$Reset,
    [switch]$Finalize
)

$ErrorActionPreference = 'Stop'
$modelName = 'deepseek-coder:1.3b'
$payloadRoot = Join-Path $PSScriptRoot 'payload'
$runtimeUrl = 'https://ollama.com/download/OllamaSetup.exe'
$ollamaLicenseUrl = 'https://raw.githubusercontent.com/ollama/ollama/main/LICENSE'

function Get-ModelStoreRoot {
    if (-not [string]::IsNullOrWhiteSpace($env:OLLAMA_MODELS)) {
        return $env:OLLAMA_MODELS
    }
    return Join-Path $env:USERPROFILE '.ollama\models'
}

function Get-RelativePath([string]$Root, [string]$Path) {
    return $Path.Substring($Root.TrimEnd('\').Length).TrimStart('\')
}

function Write-PayloadLicensesAndHashes([string]$Root) {
    $licenseDir = Join-Path $Root 'LICENSES'
    $hashFile = Join-Path $Root 'SHA256SUMS.txt'
    New-Item -ItemType Directory -Force -Path $licenseDir | Out-Null
    Invoke-WebRequest -Uri $ollamaLicenseUrl -OutFile (Join-Path $licenseDir 'OLLAMA-MIT.txt')
    $ollamaCommand = Get-Command ollama -ErrorAction Stop
    & $ollamaCommand.Source show $modelName --license | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $licenseDir 'DEEPSEEK-CODER.txt')
    if ($LASTEXITCODE -ne 0) {
        throw "Could not export the license for $modelName."
    }
    $hashes = Get-ChildItem -LiteralPath $Root -Recurse -File |
        Where-Object { $_.FullName -ne $hashFile } |
        Sort-Object FullName |
        ForEach-Object {
            $relative = (Get-RelativePath $Root $_.FullName).Replace('\', '/')
            "{0} *{1}" -f (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash, $relative
        }
    Set-Content -Encoding UTF8 -LiteralPath $hashFile -Value $hashes
}

if ($Finalize) {
    if (-not (Test-Path -LiteralPath $payloadRoot)) {
        throw "Cannot finalize a missing bundle payload: $payloadRoot"
    }
    Write-PayloadLicensesAndHashes $payloadRoot
    Write-Output "Finalized $modelName bundle at $payloadRoot."
    exit 0
}

if ($Reset -and (Test-Path -LiteralPath $payloadRoot)) {
    Remove-Item -LiteralPath $payloadRoot -Recurse -Force
}
if (Test-Path -LiteralPath $payloadRoot) {
    throw "Bundle payload already exists: $payloadRoot. Run with -Reset only when you intend to recreate it."
}

$ollamaCommand = Get-Command ollama -ErrorAction Stop
& $ollamaCommand.Source pull $modelName
if ($LASTEXITCODE -ne 0) {
    throw "Ollama could not download $modelName."
}

$modelStore = Get-ModelStoreRoot
$manifestRoot = Join-Path $modelStore 'manifests'
$manifest = Get-ChildItem -LiteralPath $manifestRoot -Recurse -File |
    Where-Object { $_.FullName -match 'deepseek-coder[\\/]1\.3b$' } |
    Select-Object -First 1
if (-not $manifest) {
    throw "Ollama downloaded $modelName but its manifest was not found in $manifestRoot."
}

$runtimePath = Join-Path $payloadRoot 'ollama\OllamaSetup.exe'
$modelPayload = Join-Path $payloadRoot 'models'
$licenseDir = Join-Path $payloadRoot 'LICENSES'
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $runtimePath), $modelPayload, $licenseDir | Out-Null

Invoke-WebRequest -Uri $runtimeUrl -OutFile $runtimePath
$signature = Get-AuthenticodeSignature -LiteralPath $runtimePath
if ($signature.Status -ne 'Valid') {
    throw "The downloaded OllamaSetup.exe is not a valid signed file: $($signature.Status)."
}

$manifestTarget = Join-Path $modelPayload (Get-RelativePath $modelStore $manifest.FullName)
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $manifestTarget) | Out-Null
Copy-Item -LiteralPath $manifest.FullName -Destination $manifestTarget

$model = Get-Content -Raw -Encoding UTF8 -LiteralPath $manifest.FullName | ConvertFrom-Json
$digests = @($model.config.digest) + @($model.layers | ForEach-Object { $_.digest }) |
    Where-Object { $_ -is [string] -and $_.StartsWith('sha256:') } |
    Select-Object -Unique
if ($digests.Count -eq 0) {
    throw "The $modelName manifest does not reference any blobs."
}
foreach ($digest in $digests) {
    $blobName = $digest -replace ':', '-'
    $sourceBlob = Join-Path $modelStore "blobs\$blobName"
    if (-not (Test-Path -LiteralPath $sourceBlob)) {
        throw "Missing source blob for ${digest}: $sourceBlob"
    }
    $targetBlob = Join-Path $modelPayload "blobs\$blobName"
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $targetBlob) | Out-Null
    Copy-Item -LiteralPath $sourceBlob -Destination $targetBlob
}

Write-PayloadLicensesAndHashes $payloadRoot

Write-Output "Prepared $modelName bundle at $payloadRoot with $($digests.Count) model blobs."
