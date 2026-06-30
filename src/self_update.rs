use std::cmp::Ordering;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

const REPO: &str = "TW-RF54732/onesource";

pub fn run() -> Result<()> {
    let current_exe = env::current_exe().context("Failed to locate current executable")?;
    let temp_path = temp_download_path(&current_exe)?;
    let asset_name = asset_name()?;
    let current_version = env!("CARGO_PKG_VERSION");

    println!("Fetching latest release info from GitHub...");
    println!("Target: {}", current_exe.display());
    println!("Current version: {}", current_version);

    let latest_version = fetch_latest_version()?;
    println!("Latest version:  {}", latest_version);

    if !version_is_newer(&latest_version, current_version) {
        println!("Already up to date.");
        return Ok(());
    }

    println!("Asset:  {}", asset_name);

    download_latest_asset(asset_name, &temp_path)?;

    #[cfg(windows)]
    {
        replace_after_exit(&current_exe, &temp_path)?;
        println!("Update downloaded. onesource will replace itself after this process exits.");
        println!("Run `onesource --version` in a new terminal to verify the update.");
    }

    #[cfg(not(windows))]
    {
        replace_now(&current_exe, &temp_path)?;
        println!("Updated onesource at {}", current_exe.display());
    }

    Ok(())
}

#[cfg(windows)]
fn fetch_latest_version() -> Result<String> {
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/{repo}/releases/latest'
$release.tag_name
"#,
        repo = REPO,
    );

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .context("PowerShell failed to fetch the latest release")?;

    parse_command_output(output, "PowerShell failed to fetch the latest release")
}

#[cfg(not(windows))]
fn fetch_latest_version() -> Result<String> {
    let api = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let script = format!(
        "curl -fsSL '{}' | grep '\"tag_name\"' | cut -d '\"' -f 4 | head -n 1",
        shell_quote(&api),
    );

    let output = Command::new("sh")
        .args(["-c", &script])
        .output()
        .context("curl failed to fetch the latest release")?;

    parse_command_output(output, "curl failed to fetch the latest release")
}

fn parse_command_output(output: std::process::Output, error_context: &str) -> Result<String> {
    if !output.status.success() {
        return Err(anyhow!("{} (exit code: {})", error_context, output.status));
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        Err(anyhow!("{} returned an empty response", error_context))
    } else {
        Ok(value)
    }
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(latest), Some(current)) => latest.cmp(&current) == Ordering::Greater,
        _ => normalize_version(latest) != normalize_version(current),
    }
}

fn parse_version(value: &str) -> Option<Vec<u64>> {
    let normalized = normalize_version(value);
    let stable = normalized.split_once('-').map_or(normalized, |(v, _)| v);
    stable.split('.').map(|part| part.parse().ok()).collect()
}

fn normalize_version(value: &str) -> &str {
    value.trim().trim_start_matches('v').trim_start_matches('V')
}

fn temp_download_path(current_exe: &Path) -> Result<PathBuf> {
    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow!("Executable has no parent directory"))?;
    let file_name = if cfg!(windows) {
        format!("onesource-update-{}.exe", std::process::id())
    } else {
        format!("onesource-update-{}", std::process::id())
    };
    Ok(parent.join(file_name))
}

fn asset_name() -> Result<&'static str> {
    match env::consts::OS {
        "windows" => Ok("onesource.exe"),
        "linux" => Ok("onesource-linux"),
        "macos" => Ok("onesource-macos"),
        other => Err(anyhow!("Unsupported OS for self-update: {}", other)),
    }
}

#[cfg(windows)]
fn download_latest_asset(asset_name: &str, output_path: &Path) -> Result<()> {
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/{repo}/releases/latest'
$asset = $release.assets | Where-Object {{ $_.name -eq '{asset}' }} | Select-Object -First 1
if (-not $asset) {{ throw "Could not find asset: {asset}" }}
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile '{output}'
"#,
        repo = REPO,
        asset = asset_name,
        output = ps_quote(output_path),
    );

    run_command(
        Command::new("powershell").args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ]),
        "PowerShell failed to download the latest release",
    )
}

#[cfg(not(windows))]
fn download_latest_asset(asset_name: &str, output_path: &Path) -> Result<()> {
    let api = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let pattern = format!("browser_download_url.*{}", asset_name);
    let output = output_path.display().to_string();
    let script = format!(
        "set -e\nurl=$(curl -fsSL '{api}' | grep '{pattern}' | cut -d '\"' -f 4 | head -n 1)\nif [ -z \"$url\" ]; then echo 'Could not find asset: {asset}' >&2; exit 1; fi\ncurl -fL -o '{output}' \"$url\"\nchmod +x '{output}'",
        api = api,
        pattern = shell_quote(&pattern),
        asset = asset_name,
        output = shell_quote(&output),
    );

    run_command(
        Command::new("sh").args(["-c", &script]),
        "curl failed to download the latest release",
    )
}

#[cfg(windows)]
fn replace_after_exit(target: &Path, downloaded: &Path) -> Result<()> {
    let pid = std::process::id();
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$process = Get-Process -Id {pid} -ErrorAction SilentlyContinue
if ($process) {{ Wait-Process -Id {pid} }}
Move-Item -Force -LiteralPath '{downloaded}' -Destination '{target}'
"#,
        pid = pid,
        downloaded = ps_quote(downloaded),
        target = ps_quote(target),
    );

    Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .spawn()
        .context("Failed to start background PowerShell updater")?;

    Ok(())
}

#[cfg(not(windows))]
fn replace_now(target: &Path, downloaded: &Path) -> Result<()> {
    std::fs::rename(downloaded, target).with_context(|| {
        format!(
            "Failed to replace {} with {}",
            target.display(),
            downloaded.display()
        )
    })
}

fn run_command(command: &mut Command, error_context: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| error_context.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("{} (exit code: {})", error_context, status))
    }
}

#[cfg(windows)]
fn ps_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}

#[cfg(not(windows))]
fn shell_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_matches_supported_platforms() {
        assert!(matches!(
            asset_name().unwrap(),
            "onesource.exe" | "onesource-linux" | "onesource-macos"
        ));
    }

    #[test]
    fn temp_download_path_stays_next_to_current_exe() {
        let exe = if cfg!(windows) {
            PathBuf::from(r"C:\tools\onesource.exe")
        } else {
            PathBuf::from("/usr/local/bin/onesource")
        };

        let temp = temp_download_path(&exe).unwrap();
        assert_eq!(temp.parent(), exe.parent());
        assert!(temp
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("onesource-update-"));
    }

    #[test]
    fn version_compare_detects_newer_versions() {
        assert!(version_is_newer("v3.3.1", "3.3.0"));
        assert!(version_is_newer("4.0.0", "3.9.9"));
        assert!(!version_is_newer("v3.3.0", "3.3.0"));
        assert!(!version_is_newer("3.2.9", "3.3.0"));
    }
}
