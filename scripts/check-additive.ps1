#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Enforces Rule R1 (Additive-Only Compliance) against upstream SolidCraft.
#>
param(
    [string]$UpstreamRef = "upstream/main"
)

$ErrorActionPreference = "Stop"
Write-Host "=== CADense Rule R1 Additive-Only Compliance Check ===" -ForegroundColor Cyan

$remotes = git remote
if ($remotes -notcontains "upstream") {
    Write-Warning "Remote 'upstream' not configured yet. Run: git remote add upstream https://github.com/tangency-labs/solidcraft.git"
    exit 0
}

$mergeBase = git merge-base HEAD $UpstreamRef
$diffFiles = git diff --name-status $mergeBase HEAD

$violations = @()
foreach ($line in $diffFiles) {
    if (-not $line) { continue }
    $parts = $line -split "\s+", 2
    $status = $parts[0]
    $path = $parts[1]

    $isCadensePath = (
        $path.StartsWith("crates/cadense-") -or
        $path.StartsWith("apps/cadense-") -or
        $path.StartsWith("scripts/") -or
        $path.StartsWith(".antigravity/") -or
        $path.StartsWith("references/") -or
        $path.StartsWith("docs/cadense") -or
        $path -eq "CADENSE_STATUS.md" -or
        $path -eq "Cargo.toml" -or
        $path -eq "Cargo.lock"
    )

    if (-not $isCadensePath) {
        if ($status -eq "D") {
            $violations += "DELETED upstream file: $path"
        } else {
            $fileContent = Get-Content $path -Raw -ErrorAction SilentlyContinue
            if ($fileContent -and ($fileContent -notmatch "CADENSE-HOOK")) {
                $violations += "MODIFIED upstream file without [CADENSE-HOOK] marker: $path"
            }
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "❌ Rule R1 Violations Detected:" -ForegroundColor Red
    foreach ($v in $violations) {
        Write-Host "  - $v" -ForegroundColor Red
    }
    exit 1
}

Write-Host "✅ Rule R1 Check Passed: All CADense extensions are strictly additive!" -ForegroundColor Green
