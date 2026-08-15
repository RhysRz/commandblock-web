[CmdletBinding()]
param(
  [string]$PreviousTag = "",
  [string]$BuildId = "",
  [string]$Repository = "",
  [string[]]$Subject = @()
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

function Convert-CommitSubjectToThai([string]$CommitSubject) {
  $line = if ($null -eq $CommitSubject) { "" } else { $CommitSubject.Trim() }
  if ([string]::IsNullOrWhiteSpace($line)) { return $null }
  $match = [regex]::Match($line, '^(?<type>feat|fix|perf|refactor|docs|test|ci|build|chore)(?:\([^)]+\))?!?:\s*(?<text>.+)$', 'IgnoreCase')
  if (!$match.Success) { return $null }

  $kind = $match.Groups['type'].Value.ToLowerInvariant()
  if ($kind -in @('docs', 'test', 'ci', 'build', 'chore')) { return $null }
  $text = $match.Groups['text'].Value.ToLowerInvariant()
  $detail = if ($text -match 'native confirm|confirmation|confirm dialog') { 'หน้าต่างยืนยันการทำรายการ' }
    elseif ($text -match 'download|install|updat') { 'การดาวน์โหลดและติดตั้งอัปเดต' }
    elseif ($text -match 'session.*pin|pin.*session|sticky pin') { 'การปักหมุดข้อความและ SESSION' }
    elseif ($text -match 'session') { 'การจัดการ SESSION' }
    elseif ($text -match 'plugin') { 'หน้าปลั๊กอินและความสามารถเสริม' }
    elseif ($text -match 'desktop connector|connector') { 'Desktop Connector' }
    elseif ($text -match 'remote') { 'Remote PC' }
    elseif ($text -match 'mobile') { 'หน้าจอบนมือถือ' }
    elseif ($text -match 'backup') { 'การสำรองและกู้คืนข้อมูล' }
    elseif ($text -match 'login|auth') { 'การเข้าสู่ระบบ' }
    else { 'ความสามารถของ CommandBlock' }

  $prefix = switch ($kind) {
    'feat' { 'เพิ่ม' }
    'fix' { 'แก้ไข' }
    'perf' { 'ปรับให้เร็วขึ้น' }
    default { 'ปรับปรุง' }
  }
  return "$prefix$detail"
}

if (!$Subject -or $Subject.Count -eq 0) {
  if ([string]::IsNullOrWhiteSpace($BuildId)) { throw 'BuildId is required when Subject is not supplied' }
  $range = if ([string]::IsNullOrWhiteSpace($PreviousTag)) { $BuildId } else { "$PreviousTag..$BuildId" }
  $Subject = @(git log $range --format=%s -- src assets Cargo.toml Cargo.lock build.rs)
}

$bullets = @($Subject | ForEach-Object { Convert-CommitSubjectToThai $_ } | Where-Object { $_ } | Select-Object -Unique | ForEach-Object { "- $_" })
if ($bullets.Count -eq 0) { $bullets = @('- ปรับปรุงความเสถียรและประสิทธิภาพของ CommandBlock') }

$tag = if ([string]::IsNullOrWhiteSpace($BuildId)) { '' } else { "build-$BuildId" }
$fullChangelog = if (![string]::IsNullOrWhiteSpace($Repository) -and ![string]::IsNullOrWhiteSpace($PreviousTag) -and $tag) {
  "https://github.com/$Repository/compare/$PreviousTag...$tag"
} elseif (![string]::IsNullOrWhiteSpace($Repository) -and $tag) {
  "https://github.com/$Repository/commits/$tag"
} else { '' }

$lines = @('## สรุปการอัปเดต', '') + $bullets + @('', '## รายละเอียดการเปลี่ยนแปลง', '')
if ($fullChangelog) { $lines += "**Full Changelog**: $fullChangelog" }
else { $lines += 'รายละเอียดการเปลี่ยนแปลงอยู่ใน GitHub release นี้' }
$lines -join "`n"
