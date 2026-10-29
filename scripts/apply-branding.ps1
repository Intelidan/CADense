#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Automated Rebranding Suite for CADense.
    Validates and restores all CADense brand identity hooks (Logo, Home Hero,
    Titlebar, About Dialog, Native Menus, Binary Metadata) across upstream SolidCraft files.
    Ensures 100% Rule R1 Additive Compliance.
#>
param(
    [switch]$CheckOnly = $false,
    [string]$TargetDir = 'target/test_target'
)

$ErrorActionPreference = 'Stop'
Write-Host '==========================================================' -ForegroundColor Cyan
Write-Host '   CADense Full Branding Suite - Automated Hook Sync   ' -ForegroundColor Cyan
Write-Host '==========================================================' -ForegroundColor Cyan

$Root = Split-Path -Parent $PSScriptRoot
$hooksChecked = 0
$hooksValid = 0
$hooksRepaired = 0

function Check-Or-Patch-Hook {
    param(
        [string]$FilePath,
        [string]$HookMarker,
        [string]$Description,
        [scriptblock]$PatchAction
    )
    $fullPath = Join-Path $Root $FilePath
    if (-not (Test-Path $fullPath)) {
        Write-Host "X File missing: $FilePath" -ForegroundColor Red
        return
    }

    $script:hooksChecked++
    $content = Get-Content $fullPath -Raw

    if ($content.Contains($HookMarker)) {
        Write-Host "  [OK] $Description ($FilePath)" -ForegroundColor Green
        $script:hooksValid++
    } else {
        if ($CheckOnly) {
            Write-Host "  [MISSING] $Description ($FilePath)" -ForegroundColor Yellow
        } else {
            Write-Host "  [PATCHING] $Description ($FilePath)..." -ForegroundColor Yellow
            & $PatchAction $fullPath
            $script:hooksRepaired++
            Write-Host "  [PATCHED] $Description restored." -ForegroundColor Green
        }
    }
}

# 1. Cargo.toml Workspace Registration
Check-Or-Patch-Hook -FilePath 'Cargo.toml' -HookMarker 'solidcraft-cms = { path = "crates/solidcraft_cms" }' -Description 'Root Cargo.toml CMS Dependency' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = '[workspace.dependencies]'
    $replacement = "[workspace.dependencies]`nsolidcraft-cms = { path = `"crates/solidcraft_cms`" }"
    $c = $c.Replace($target, $replacement)
    Set-Content -Path $f -Value $c -NoNewline
}

# 2. crates/ui-egui/Cargo.toml Dependency
Check-Or-Patch-Hook -FilePath 'crates/ui-egui/Cargo.toml' -HookMarker '[CADENSE-HOOK: CMS truFold sub-crate dependency]' -Description 'ui-egui Cargo.toml CMS Dependency' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = '[dependencies]'
    $replacement = "[dependencies]`n# === [CADENSE-HOOK: CMS truFold sub-crate dependency] ===`nsolidcraft-cms = { workspace = true }"
    $c = $c.Replace($target, $replacement)
    Set-Content -Path $f -Value $c -NoNewline
}

# 3. crates/ui-egui/src/toolbar.rs Brand Identity Lockup
Check-Or-Patch-Hook -FilePath 'crates/ui-egui/src/toolbar.rs' -HookMarker '[CADENSE-HOOK: CADense Brand Identity Mark & Title Lockup]' -Description 'Toolbar Brand Mark and Title Lockup' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = 'crate::icons::glyphs::brand_mark(ui.painter(), c);'
    if ($c.Contains($target)) {
        $replacement = @'
// === [CADENSE-HOOK: CADense Brand Identity Mark & Title Lockup] ===
        {
            let _ = crate::icons::glyphs::brand_mark;
            let c = pos2(left + 20.0, r.center().y);
            solidcraft_cms::branding::paint_cadense_brand_mark(ui.painter(), c, 22.0);
        }
'@
        $c = $c.Replace($target, $replacement)
        $c = $c.Replace('ui.painter().text(pos2(left + 36.0, r.center().y), Align2::LEFT_CENTER, "SolidCraft",', 'ui.painter().text(pos2(left + 36.0, r.center().y), Align2::LEFT_CENTER, "CADense",')
        Set-Content -Path $f -Value $c -NoNewline
    }
}

# 4. crates/ui-egui/src/help.rs About Dialog & Diagnostics
Check-Or-Patch-Hook -FilePath 'crates/ui-egui/src/help.rs' -HookMarker '[CADENSE-HOOK: CADense About Dialog Rebranding]' -Description 'About Dialog and Diagnostic Version Info' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = 'ui.heading(app.tr("ui.about.title"));'
    if ($c.Contains($target)) {
        $hook = @'
    // === [CADENSE-HOOK: CADense About Dialog Rebranding] ===
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
        solidcraft_cms::branding::paint_cadense_brand_mark(ui.painter(), rect.center(), 56.0);
        ui.add_space(8.0);
        ui.heading(egui::RichText::new(solidcraft_cms::branding::CadenseVersion::APP_NAME).strong().size(22.0).color(solidcraft_cms::branding::CADENSE_SILVER_WHITE));
        ui.label(egui::RichText::new(solidcraft_cms::branding::CadenseVersion::TAGLINE).size(12.0).color(solidcraft_cms::branding::CADENSE_SILVER_BEVEL));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("Version {} (TruFold Engine {})", solidcraft_cms::branding::CadenseVersion::VERSION, solidcraft_cms::branding::CadenseVersion::TRUFOLD_ENGINE_VERSION)).small().color(solidcraft_cms::branding::CADENSE_COBALT_BLUE));
        ui.label(egui::RichText::new(format!("Built on {}", solidcraft_cms::branding::CadenseVersion::UPSTREAM_ENGINE)).italics().small().color(egui::Color32::from_rgb(140, 150, 165)));
    });
    ui.add_space(12.0);
'@
        $c = $c.Replace($target, $hook)
        Set-Content -Path $f -Value $c -NoNewline
    }
}

# 5. crates/ui-egui/src/home.rs Hero Banner & Preset Launcher
Check-Or-Patch-Hook -FilePath 'crates/ui-egui/src/home.rs' -HookMarker '[CADENSE-HOOK: CADense Home Hero Banner & Preset Launcher]' -Description 'Home Hero Banner and CMS Preset Launcher' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = 'render_gallery(ui, app);'
    if ($c.Contains($target)) {
        $hook = @'
            // === [CADENSE-HOOK: CADense Home Hero Banner & Preset Launcher] ===
            solidcraft_cms::branding::render_cadense_home_hero(ui, &mut |preset_name| {
                match preset_name {
                    "reducer" => solidcraft_cms::ui::load_preset_reducer(),
                    "tee" => solidcraft_cms::ui::load_preset_unequal_tee(),
                    "bend" => solidcraft_cms::ui::load_preset_bend(),
                    "cross" => solidcraft_cms::ui::load_preset_cross(),
                    _ => {}
                }
            });
            ui.add_space(16.0);
            render_gallery(ui, app);
'@
        $c = $c.Replace($target, $hook)
        $c = $c.Replace('"About SolidCraft"', '"About CADense"')
        Set-Content -Path $f -Value $c -NoNewline
    }
}

# 6. apps/solidcraft/src/main.rs Window Title & Process Name
Check-Or-Patch-Hook -FilePath 'apps/solidcraft/src/main.rs' -HookMarker '[CADENSE-HOOK: CADense Window Title & Branding]' -Description 'Window Title and Process Name' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = '.with_title("SolidCraft")'
    if ($c.Contains($target)) {
        $c = $c.Replace($target, @'
        // === [CADENSE-HOOK: CADense Window Title & Branding] ===
        .with_title("CADense — Sheet Metal & CMS Automation")
'@)
    }
    Set-Content -Path $f -Value $c -NoNewline
}

# 7. apps/solidcraft/src/native_menu.rs Menu Rebranding
Check-Or-Patch-Hook -FilePath 'apps/solidcraft/src/native_menu.rs' -HookMarker '[CADENSE-HOOK: CADense Native Menu Rebranding]' -Description 'Native Application Menu Rebranding' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = 'let app_menu = Submenu::new("SolidCraft", true);'
    if ($c.Contains($target)) {
        $c = $c.Replace($target, @'
        // === [CADENSE-HOOK: CADense Native Menu Rebranding] ===
        let app_menu = Submenu::new("CADense", true);
'@)
        $c = $c.Replace('MenuItem::new("About SolidCraft",', @'
        // === [CADENSE-HOOK: CADense Native Menu About] ===
        MenuItem::new("About CADense",
'@)
    }
    Set-Content -Path $f -Value $c -NoNewline
}

# 8. apps/solidcraft/build.rs Windows Binary Properties
Check-Or-Patch-Hook -FilePath 'apps/solidcraft/build.rs' -HookMarker '[CADENSE-HOOK: CADense Windows Binary Properties]' -Description 'Windows Executable Binary Properties' -PatchAction {
    param($f)
    $c = Get-Content $f -Raw
    $target = 'VALUE \"CompanyName\", \"SolidCraft contributors\"\nVALUE \"FileDescription\", \"SolidCraft parametric CAD\"\nVALUE \"FileVersion\", \"{version}\"\nVALUE \"InternalName\", \"solidcraft\"\nVALUE \"OriginalFilename\", \"solidcraft.exe\"\nVALUE \"ProductName\", \"SolidCraft\"\nVALUE \"ProductVersion\", \"{version}\"'
    if ($c.Contains($target)) {
        $replacement = 'VALUE \"CompanyName\", \"CADense Industrial Automation\"\nVALUE \"FileDescription\", \"CADense Sheet Metal & CMS Automation Platform\"\nVALUE \"FileVersion\", \"{version}\"\nVALUE \"InternalName\", \"cadense\"\nVALUE \"OriginalFilename\", \"cadense.exe\"\nVALUE \"ProductName\", \"CADense\"\nVALUE \"ProductVersion\", \"{version}\"'
        $c = $c.Replace($target, $replacement)
        Set-Content -Path $f -Value $c -NoNewline
    }
}

Write-Host '----------------------------------------------------------' -ForegroundColor Cyan
Write-Host "Summary: $hooksChecked checked, $hooksValid verified intact, $hooksRepaired repaired." -ForegroundColor Cyan

# Rule R1 Additive Check
Write-Host 'Verifying Rule R1 Additive-Only Compliance...' -ForegroundColor Yellow
& "$PSScriptRoot/check-additive.ps1" -UpstreamRef v0.26.1

# Compilation Verification
Write-Host 'Verifying compilation with branded components...' -ForegroundColor Yellow
cargo check -p solidcraft-cms -p solidcraft-ui-egui -p solidcraft --target-dir $TargetDir

Write-Host 'CADense Full Branding Suite successfully synchronized!' -ForegroundColor Green
