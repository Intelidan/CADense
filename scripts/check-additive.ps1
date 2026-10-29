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
        $path.StartsWith("crates/solidcraft_cms") -or
        $path.StartsWith("apps/cadense-") -or
        $path.StartsWith("scripts/") -or
        $path.StartsWith(".antigravity/") -or
        $path.StartsWith(".agent/") -or
        $path.StartsWith("references/") -or
        $path.StartsWith("docs/") -or
        $path.StartsWith("assets/dxf/") -or
        $path.StartsWith("assets/templates/") -or
        $path.StartsWith("drawings/cms_in_progress/") -or
        $path -eq "CADENSE_STATUS.md" -or
        $path -eq "CMS_TRUFOLD_DUAL_RIBBON_IMPLEMENTATION_PLAN.md" -or
        $path -eq "dxf_analysis_report.md" -or
        $path -eq "Cargo.toml" -or
        $path -eq "Cargo.lock" -or
        $path -eq "AGENTS.md" -or
        $path -eq ".cargo/config.toml"
    )

    if (-not $isCadensePath) {
        if ($status -eq "D") {
            $violations += "DELETED upstream file: $path"
        } elseif ($status -eq "M") {
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
