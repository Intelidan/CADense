#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Safely fetches and merges a new SolidCraft release tag or branch into CADense.
#>
param(
    [string]$TargetTag = "v0.23.1"
)

$ErrorActionPreference = "Stop"
Write-Host "=== Syncing CADense with Upstream SolidCraft ($TargetTag) ===" -ForegroundColor Cyan

$remotes = git remote
if ($remotes -notcontains "upstream") {
    Write-Host "Adding official SolidCraft remote as 'upstream'..."
    git remote add upstream https://github.com/tangency-labs/solidcraft.git
}

git fetch upstream --tags

$syncBranch = "sync/$TargetTag"
Write-Host "Creating safety sync branch: $syncBranch" -ForegroundColor Yellow
git checkout -B $syncBranch

Write-Host "Merging $TargetTag into $syncBranch..."
git merge $TargetTag --no-edit

Write-Host "Running Rule R1 Additive Check..."
& "$PSScriptRoot/check-additive.ps1" -UpstreamRef $TargetTag

Write-Host "Running cargo check on CADense crates..."
cargo check -p cadense-dsf -p cadense-ui

Write-Host "✅ Sync with SolidCraft $TargetTag succeeded on branch '$syncBranch'!" -ForegroundColor Green
Write-Host "To finalize: git checkout main && git merge $syncBranch && git push origin main"
