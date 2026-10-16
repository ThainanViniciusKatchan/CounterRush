use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell;

// -----------------------------------------------------------------------------
// Discord
// -----------------------------------------------------------------------------

pub struct DiscordGpuAccelTweak;

impl Tweak for DiscordGpuAccelTweak {
    fn id(&self) -> &'static str {
        "gpu_accel_discord"
    }

    fn name(&self) -> &'static str {
        "Desativar Aceleração de GPU no Discord"
    }

    fn description(&self) -> &'static str {
        "Desativa a aceleração de hardware nas configurações do Discord para liberar VRAM e ciclos de GPU para o CS2."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = r#"
            $config = "$env:APPDATA\discord\settings.json"
            if (Test-Path $config) {
                $json = Get-Content $config -Raw | ConvertFrom-Json
                if ($json.enableHardwareAcceleration -eq $false) { "Applied" } else { "Default" }
            } else { "Unknown" }
        "#;
        match run_powershell(script) {
            Ok(output) if output.contains("Applied") => Ok(TweakStatus::Applied),
            Ok(output) if output.contains("Default") => Ok(TweakStatus::Default),
            _ => Ok(TweakStatus::Unknown),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_discord_gpu_accel()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        enable_discord_gpu_accel()
    }
}

pub fn disable_discord_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "Discord" -ErrorAction SilentlyContinue | Stop-Process -Force
        $config = "$env:APPDATA\discord\settings.json"
        if (Test-Path $config) {
            $json = Get-Content $config -Raw | ConvertFrom-Json
            $json | Add-Member -NotePropertyName "enableHardwareAcceleration" -NotePropertyValue $false -Force
            $json | ConvertTo-Json | Set-Content $config
        }
        Start-Process "$env:LOCALAPPDATA\Discord\Update.exe" -ArgumentList "--processStart Discord.exe"
    "#;
    run_powershell(script)?;
    Ok(())
}

pub fn enable_discord_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "Discord" -ErrorAction SilentlyContinue | Stop-Process -Force
        $config = "$env:APPDATA\discord\settings.json"
        if (Test-Path $config) {
            $json = Get-Content $config -Raw | ConvertFrom-Json
            $json | Add-Member -NotePropertyName "enableHardwareAcceleration" -NotePropertyValue $true -Force
            $json | ConvertTo-Json | Set-Content $config
        }
        Start-Process "$env:LOCALAPPDATA\Discord\Update.exe" -ArgumentList "--processStart Discord.exe"
    "#;
    run_powershell(script)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Steam
// -----------------------------------------------------------------------------

pub struct SteamGpuAccelTweak;

impl Tweak for SteamGpuAccelTweak {
    fn id(&self) -> &'static str {
        "gpu_accel_steam"
    }

    fn name(&self) -> &'static str {
        "Desativar Aceleração de GPU na Steam"
    }

    fn description(&self) -> &'static str {
        "Desativa a aceleração de GPU na renderização web da Steam (steamwebhelper) para reduzir consumo de VRAM e GPU."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = r#"
            $val = (Get-ItemProperty -Path "HKCU:\Software\Valve\Steam" -Name "GPUAccelWebViewsV3" -ErrorAction SilentlyContinue).GPUAccelWebViewsV3
            if ($val -eq 0) { "Applied" } else { "Default" }
        "#;
        match run_powershell(script) {
            Ok(output) if output.contains("Applied") => Ok(TweakStatus::Applied),
            _ => Ok(TweakStatus::Default),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_steam_gpu_accel()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        enable_steam_gpu_accel()
    }
}

pub fn disable_steam_gpu_accel() -> Result<(), TweakError> {
    if windows_api::is_process_running("cs2").unwrap_or(false) {
        return Err(TweakError::ValidationFailed(
            "O CS2 está em execução. Feche o jogo antes de alterar a aceleração de GPU da Steam.".to_string(),
        ));
    }
    let script = r#"
        Get-Process -Name "steam", "steamwebhelper" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $regPath = "HKCU:\Software\Valve\Steam"
        @("GPUAccelWebViewsV3", "GPUAccelWebViews", "GPUAcceleratedWebViews") | ForEach-Object {
            New-ItemProperty -Path $regPath -Name $_ -Value 0 -PropertyType DWord -Force | Out-Null
        }
        New-ItemProperty -Path $regPath -Name "SmoothScrollWebViews" -Value 0 -PropertyType DWord -Force | Out-Null
        $steamPath = (Get-ItemProperty -Path $regPath -Name "SteamPath" -ErrorAction SilentlyContinue).SteamPath
        if ($steamPath) {
            Get-ChildItem -Path "$steamPath\userdata\*\config\localconfig.vdf" -ErrorAction SilentlyContinue | ForEach-Object {
                $content = Get-Content $_.FullName -Raw
                $content = $content -replace '"SmoothScrollWebViews"\s+"[01]"', '"SmoothScrollWebViews" "0"'
                $content = $content -replace '"GPUAccelWebViewsV3"\s+"[01]"', '"GPUAccelWebViewsV3" "0"'
                $content = $content -replace '"GPUAccelWebViews"\s+"[01]"', '"GPUAccelWebViews" "0"'
                Set-Content -Path $_.FullName -Value $content
            }
        }
        $steamExe = (Get-ItemProperty -Path $regPath -Name "SteamExe" -ErrorAction SilentlyContinue).SteamExe
        if ($steamExe -and (Test-Path $steamExe)) {
            Start-Process $steamExe
        } else {
            Start-Process "steam://open/main"
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

pub fn enable_steam_gpu_accel() -> Result<(), TweakError> {
    if windows_api::is_process_running("cs2").unwrap_or(false) {
        return Err(TweakError::ValidationFailed(
            "O CS2 está em execução. Feche o jogo antes de alterar a aceleração de GPU da Steam.".to_string(),
        ));
    }
    let script = r#"
        Get-Process -Name "steam", "steamwebhelper" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $regPath = "HKCU:\Software\Valve\Steam"
        @("GPUAccelWebViewsV3", "GPUAccelWebViews", "GPUAcceleratedWebViews") | ForEach-Object {
            New-ItemProperty -Path $regPath -Name $_ -Value 1 -PropertyType DWord -Force | Out-Null
        }
        New-ItemProperty -Path $regPath -Name "SmoothScrollWebViews" -Value 1 -PropertyType DWord -Force | Out-Null
        $steamPath = (Get-ItemProperty -Path $regPath -Name "SteamPath" -ErrorAction SilentlyContinue).SteamPath
        if ($steamPath) {
            Get-ChildItem -Path "$steamPath\userdata\*\config\localconfig.vdf" -ErrorAction SilentlyContinue | ForEach-Object {
                $content = Get-Content $_.FullName -Raw
                $content = $content -replace '"SmoothScrollWebViews"\s+"[01]"', '"SmoothScrollWebViews" "1"'
                $content = $content -replace '"GPUAccelWebViewsV3"\s+"[01]"', '"GPUAccelWebViewsV3" "1"'
                $content = $content -replace '"GPUAccelWebViews"\s+"[01]"', '"GPUAccelWebViews" "1"'
                Set-Content -Path $_.FullName -Value $content
            }
        }
        $steamExe = (Get-ItemProperty -Path $regPath -Name "SteamExe" -ErrorAction SilentlyContinue).SteamExe
        if ($steamExe -and (Test-Path $steamExe)) {
            Start-Process $steamExe
        } else {
            Start-Process "steam://open/main"
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Chromium (Chrome, Edge, Brave)
// -----------------------------------------------------------------------------

pub struct ChromiumGpuAccelTweak;

impl Tweak for ChromiumGpuAccelTweak {
    fn id(&self) -> &'static str {
        "gpu_accel_chromium"
    }

    fn name(&self) -> &'static str {
        "Desativar Aceleração de GPU nos Navegadores Chromium"
    }

    fn description(&self) -> &'static str {
        "Desativa a aceleração de hardware no Chrome, Edge e Brave para evitar retenção de VRAM."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = r#"
            $arquivos = @(
                "$env:LOCALAPPDATA\Google\Chrome\User Data\Local State",
                "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Local State",
                "$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data\Local State"
            )
            $found = $false
            $allDisabled = $true
            foreach ($arquivo in $arquivos) {
                if (Test-Path $arquivo) {
                    $found = $true
                    $json = Get-Content $arquivo -Raw -Encoding UTF8 | ConvertFrom-Json
                    if ($json.hardware_acceleration_mode.enabled -ne $false) {
                        $allDisabled = $false
                    }
                }
            }
            if (-not $found) { "Unknown" }
            elseif ($allDisabled) { "Applied" }
            else { "Default" }
        "#;
        match run_powershell(script) {
            Ok(output) if output.contains("Applied") => Ok(TweakStatus::Applied),
            Ok(output) if output.contains("Default") => Ok(TweakStatus::Default),
            _ => Ok(TweakStatus::Unknown),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_chromium_gpu_accel()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        enable_chromium_gpu_accel()
    }
}

pub fn disable_chromium_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "chrome", "msedge", "brave" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $arquivos = @(
            "$env:LOCALAPPDATA\Google\Chrome\User Data\Local State",
            "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Local State",
            "$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data\Local State"
        )
        foreach ($arquivo in $arquivos) {
            if (Test-Path $arquivo) {
                $json = Get-Content $arquivo -Raw -Encoding UTF8 | ConvertFrom-Json
                $valor = [PSCustomObject]@{ enabled = $false }
                $json | Add-Member -NotePropertyName "hardware_acceleration_mode" -NotePropertyValue $valor -Force
                $json | ConvertTo-Json -Depth 100 -Compress | Set-Content $arquivo -Encoding UTF8
            }
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

pub fn enable_chromium_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "chrome", "msedge", "brave" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $arquivos = @(
            "$env:LOCALAPPDATA\Google\Chrome\User Data\Local State",
            "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Local State",
            "$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data\Local State"
        )
        foreach ($arquivo in $arquivos) {
            if (Test-Path $arquivo) {
                $json = Get-Content $arquivo -Raw -Encoding UTF8 | ConvertFrom-Json
                $valor = [PSCustomObject]@{ enabled = $true }
                $json | Add-Member -NotePropertyName "hardware_acceleration_mode" -NotePropertyValue $valor -Force
                $json | ConvertTo-Json -Depth 100 -Compress | Set-Content $arquivo -Encoding UTF8
            }
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Firefox
// -----------------------------------------------------------------------------

pub struct FirefoxGpuAccelTweak;

impl Tweak for FirefoxGpuAccelTweak {
    fn id(&self) -> &'static str {
        "gpu_accel_firefox"
    }

    fn name(&self) -> &'static str {
        "Desativar Aceleração de GPU no Firefox"
    }

    fn description(&self) -> &'static str {
        "Desativa a aceleração de hardware no Firefox (layers.acceleration.disabled) para liberar recursos para o CS2."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = r#"
            $perfis = Get-ChildItem -Path "$env:APPDATA\Mozilla\Firefox\Profiles\*\prefs.js" -ErrorAction SilentlyContinue
            if (-not $perfis) { "Unknown" }
            else {
                $applied = $true
                foreach ($arquivo in $perfis) {
                    $content = Get-Content $arquivo.FullName -Raw
                    if ($content -notmatch 'user_pref\("layers\.acceleration\.disabled",\s*true\);') {
                        $applied = $false
                    }
                }
                if ($applied) { "Applied" } else { "Default" }
            }
        "#;
        match run_powershell(script) {
            Ok(output) if output.contains("Applied") => Ok(TweakStatus::Applied),
            Ok(output) if output.contains("Default") => Ok(TweakStatus::Default),
            _ => Ok(TweakStatus::Unknown),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_firefox_gpu_accel()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        enable_firefox_gpu_accel()
    }
}

pub fn disable_firefox_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "firefox" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $perfis = Get-ChildItem -Path "$env:APPDATA\Mozilla\Firefox\Profiles\*\prefs.js" -ErrorAction SilentlyContinue
        foreach ($arquivo in $perfis) {
            $linhas = Get-Content $arquivo.FullName -Encoding UTF8 | Where-Object {
                $_ -notmatch 'browser\.preferences\.defaultPerformanceSettings\.enabled' -and
                $_ -notmatch 'layers\.acceleration\.disabled'
            }
            $linhas += 'user_pref("browser.preferences.defaultPerformanceSettings.enabled", false);'
            $linhas += 'user_pref("layers.acceleration.disabled", true);'
            Set-Content -Path $arquivo.FullName -Value $linhas -Encoding UTF8
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

pub fn enable_firefox_gpu_accel() -> Result<(), TweakError> {
    let script = r#"
        Get-Process -Name "firefox" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 1
        $perfis = Get-ChildItem -Path "$env:APPDATA\Mozilla\Firefox\Profiles\*\prefs.js" -ErrorAction SilentlyContinue
        foreach ($arquivo in $perfis) {
            $linhas = Get-Content $arquivo.FullName -Encoding UTF8 | Where-Object {
                $_ -notmatch 'browser\.preferences\.defaultPerformanceSettings\.enabled' -and
                $_ -notmatch 'layers\.acceleration\.disabled'
            }
            $linhas += 'user_pref("browser.preferences.defaultPerformanceSettings.enabled", true);'
            $linhas += 'user_pref("layers.acceleration.disabled", false);'
            Set-Content -Path $arquivo.FullName -Value $linhas -Encoding UTF8
        }
    "#;
    run_powershell(script)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Operações em Lote
// -----------------------------------------------------------------------------

pub fn disable_all_gpu_accel() -> Result<(), TweakError> {
    disable_discord_gpu_accel()?;
    disable_steam_gpu_accel()?;
    disable_chromium_gpu_accel()?;
    disable_firefox_gpu_accel()?;
    Ok(())
}

pub fn enable_all_gpu_accel() -> Result<(), TweakError> {
    enable_discord_gpu_accel()?;
    enable_steam_gpu_accel()?;
    enable_chromium_gpu_accel()?;
    enable_firefox_gpu_accel()?;
    Ok(())
}
