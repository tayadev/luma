//! `upgrade` subcommand handler for Windows

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

/// Upgrade Luma to a newer version (Windows only)
#[cfg(target_os = "windows")]
pub fn handle_upgrade(version: Option<&str>) {
    println!("Upgrading Luma...");

    // Determine installation directory
    let luma_root = if let Ok(install_dir) = env::var("LUMA_INSTALL") {
        PathBuf::from(install_dir)
    } else if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join(".luma")
    } else {
        eprintln!("Error: Could not determine installation directory");
        process::exit(1);
    };

    let luma_bin = luma_root.join("bin");
    let luma_exe = luma_bin.join("luma.exe");

    // Get current executable path to verify we're upgrading the right installation
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Could not determine current executable path: {e}");
            process::exit(1);
        }
    };

    // Warn if current exe is not in the expected location
    if current_exe != luma_exe {
        eprintln!(
            "Warning: Current executable is at {}, but will upgrade {}",
            current_exe.display(),
            luma_exe.display()
        );
    }

    // Determine if we're upgrading to nightly or a release
    let is_nightly = version == Some("nightly");

    let zip_bytes = if is_nightly {
        download_nightly_artifact()
    } else {
        download_release_artifact(version)
    };

    println!("Download complete. Extracting...");

    // Extract the zip file
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!("Error: Failed to read zip archive: {e}");
            process::exit(1);
        }
    };

    // Create bin directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&luma_bin) {
        eprintln!("Error: Failed to create bin directory: {e}");
        process::exit(1);
    }

    // Extract luma.exe from the archive
    let exe_name = "luma.exe";
    let mut found = false;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error: Failed to read file from archive: {e}");
                continue;
            }
        };

        let file_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        // Look for luma.exe in any subdirectory
        if file_path.file_name().and_then(|n| n.to_str()) == Some(exe_name) {
            found = true;

            // Need to rename current exe before replacing it
            let backup_exe = luma_bin.join("luma.exe.old");
            if luma_exe.exists()
                && let Err(e) = fs::rename(&luma_exe, &backup_exe)
            {
                eprintln!("Error: Failed to backup current executable: {e}");
                process::exit(1);
            }

            // Write new executable
            let mut outfile = match fs::File::create(&luma_exe) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: Failed to create new executable: {e}");
                    // Restore backup
                    let _ = fs::rename(&backup_exe, &luma_exe);
                    process::exit(1);
                }
            };

            if let Err(e) = std::io::copy(&mut file, &mut outfile) {
                eprintln!("Error: Failed to write new executable: {e}");
                // Restore backup
                drop(outfile);
                let _ = fs::remove_file(&luma_exe);
                let _ = fs::rename(&backup_exe, &luma_exe);
                process::exit(1);
            }

            // Remove backup on success
            let _ = fs::remove_file(&backup_exe);

            println!("✓ Successfully extracted luma.exe");
            break;
        }
    }

    if !found {
        eprintln!("Error: luma.exe not found in archive");
        process::exit(1);
    }

    // Verify the new installation
    let output = match process::Command::new(&luma_exe).arg("--version").output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Warning: Could not verify new installation: {e}");
            println!("\nUpgrade complete!");
            return;
        }
    };

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("\n✓ Luma {} upgraded successfully!", version_output.trim());
        println!("Binary location: {}", luma_exe.display());
    } else {
        eprintln!("Warning: New installation may not be working correctly");
        println!("\nUpgrade complete!");
    }
}

/// Non-Windows stub for upgrade command
#[cfg(not(target_os = "windows"))]
pub fn handle_upgrade(_version: Option<&str>) {
    eprintln!("Error: The upgrade command is currently only supported on Windows.");
    eprintln!("Please download the latest release manually from:");
    eprintln!("https://github.com/tayadev/luma/releases");
    process::exit(1);
}

#[cfg(target_os = "windows")]
fn download_release_artifact(version: Option<&str>) -> bytes::Bytes {
    use reqwest::blocking::Client;

    // Determine version string
    let version_tag = match version {
        Some(v) => {
            // Normalize version format
            if v.starts_with("v") {
                v.to_string()
            } else if v.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                format!("v{v}")
            } else {
                v.to_string()
            }
        }
        None => "latest".to_string(),
    };

    // Construct download URL
    let arch = "x64";
    let target = format!("luma-windows-{arch}");
    let base_url = "https://github.com/tayadev/luma/releases";
    let url = if version_tag == "latest" {
        format!("{base_url}/latest/download/{target}.zip")
    } else {
        format!("{base_url}/download/{version_tag}/{target}.zip")
    };

    println!("Downloading from: {url}");

    // Download the release
    let client = Client::builder()
        .user_agent("luma-upgrade")
        .build()
        .expect("Failed to create HTTP client");

    let response = match client.get(&url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: Failed to download release: {e}");
            process::exit(1);
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "Error: Failed to download release (HTTP {})",
            response.status()
        );
        process::exit(1);
    }

    match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read download: {e}");
            process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn download_nightly_artifact() -> bytes::Bytes {
    use reqwest::blocking::Client;

    println!("Fetching latest nightly build from CI...");

    let client = Client::builder()
        .user_agent("luma-upgrade")
        .build()
        .expect("Failed to create HTTP client");

    // Use nightly.link service to download artifacts from the latest successful build workflow
    let artifact_name = "luma-windows-x64";
    let url = format!("https://nightly.link/tayadev/luma/workflows/build/main/{artifact_name}.zip");

    println!("Downloading from: {url}");

    let response = match client.get(&url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: Failed to download nightly build: {e}");
            eprintln!("Make sure there is a successful build on the main branch.");
            process::exit(1);
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "Error: Failed to download nightly build (HTTP {})",
            response.status()
        );
        eprintln!("Make sure there is a successful build on the main branch.");
        process::exit(1);
    }

    match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read download: {e}");
            process::exit(1);
        }
    }
}
