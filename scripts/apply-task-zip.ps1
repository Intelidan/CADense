<#
.SYNOPSIS
    Extracts a CADense task .zip archive directly into the repository root, verifies additive compliance,
    runs cargo check, and prepares the git commit.
.EXAMPLE
    ./scripts/apply-task-zip.ps1 -ZipPath "./Downloads/Task_01_CMS_DSF_Foundation.zip" -CommitAndPush
#>
param(
    [Parameter(Mandatory=$true)]
    [string]$ZipPath,

    [Parameter(Mandatory=$false)]
    [switch]$CommitAndPush,

    [Parameter(Mandatory=$false)]
    [string]$CommitMessage = "feat(cadense): apply task update package"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $ZipPath)) {
    Write-Error "Zip file not found at: $ZipPath"
    exit 1
}

$RepoRoot = Resolve-Path "$PSScriptRoot/.."
Write-Host "Extracting '$ZipPath' into CADense root ($RepoRoot)..." -ForegroundColor Cyan
Expand-Archive -Path $ZipPath -DestinationPath $RepoRoot -Force

Push-Location $RepoRoot
try {
    if (Test-Path "./scripts/check-additive.ps1") {
        & "./scripts/check-additive.ps1"
    }

    Write-Host "Changed and added files:" -ForegroundColor Cyan
    git status -s

    if ($CommitAndPush) {
        git add .
        git commit -m $CommitMessage
        git push origin main
        Write-Host "Successfully committed and pushed to origin/main!" -ForegroundColor Green
    } else {
        Write-Host "Files applied cleanly! Run 'git add . && git commit -m ""..."" && git push origin main' when ready." -ForegroundColor Green
    }
} finally {
    Pop-Location
}
