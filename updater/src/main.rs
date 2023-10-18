use clap::{command, Parser};
use std::path::PathBuf;

use crate::github::{download_release, get_release};

mod github;

/// If you are reading this, you probably don't need to be. Dorion updates on it's own, silly!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Update Dorion
  #[arg(short = 'm', long)]
  main: Option<bool>,

  /// Path to injection folder
  #[arg(short = 'v', long)]
  vencord: Option<String>,

  /// Whether this is a local install or not
  #[arg(short = 'l', long)]
  local: Option<bool>,
}

pub fn main() {
  let args = Args::parse();

  if args.vencord.is_some() {
    if needs_to_elevate(PathBuf::from(args.vencord.clone().unwrap())) {
      println!("Elevating process...");
      elevate();
      return;
    }

    update_vencordorion(PathBuf::from(args.vencord.unwrap()));
  }

  // THis should happen second
  if args.main.is_some() {
    #[cfg(target_os = "windows")]
    if args.local.is_some() && args.local.unwrap() {
      update_main_kinda();
      return;
    }

    update_main();
  }
}

pub fn elevate() {
  // This should always be run by Dorion itself, which means it will likely not have admin perms, so we request them before anything else
  #[cfg(target_os = "windows")]
  reopen_as_elevated();

  #[cfg(not(target_os = "windows"))]
  sudo::escalate_if_needed().expect("Failed to escalate as root");
}

/**
 * Check if we can already access the folder before elevating
 */
pub fn needs_to_elevate(path: PathBuf) -> bool {
  // Write a test file to the injection folder to see if we have perms
  let mut test_file = path;
  test_file.push("test");

  let write_perms = match std::fs::write(&test_file, "") {
    Ok(()) => {
      // Delete the test file
      std::fs::remove_file(test_file).unwrap();

      true
    }
    Err(e) => {
      println!("Error writing test file: {}", e);
      false
    }
  };

  !write_perms
}

#[cfg(target_os = "windows")]
pub fn reopen_as_elevated() {
  let install = std::env::current_exe().unwrap();

  let mut binding = std::process::Command::new("powershell.exe");
  let cmd = binding.arg("-command").arg(format!(
    "Start-Process -filepath '{}' -verb runas -ArgumentList @({})",
    install.into_os_string().into_string().unwrap(),
    // get program args (without first one) and join by ,
    std::env::args()
      .skip(1)
      .map(|arg| format!("'\"{}\"'", arg))
      .collect::<Vec<String>>()
      .join(",")
  ));

  println!("Executing: {:?}", cmd);

  let mut process = cmd.spawn().unwrap();

  // Wait for the updater to finish
  process.wait().unwrap();

  std::process::exit(0);
}

pub fn update_vencordorion(path: PathBuf) {
  let release = get_release("SpikeHD", "Vencordorion");

  println!("Latest Vencordorion release: {}", release.tag_name);

  println!("Writing files to disk...");

  // Write both to disk

  download_release(
    "SpikeHD",
    "Vencordorion",
    release.tag_name.clone(),
    "browser.css",
    path.clone(),
  );

  download_release(
    "SpikeHD",
    "Vencordorion",
    release.tag_name.clone(),
    "browser.js",
    path.clone(),
  );

  // If this succeeds, write the new version to vencord.version
  let mut ven_path = path.clone();
  ven_path.push("vencord.version");

  std::fs::write(ven_path, release.tag_name).unwrap();
}

/**
 * Download the MSI and install
 */
#[cfg(target_os = "windows")]
pub fn update_main() {
  let release = get_release("SpikeHD", "Dorion");

  println!("Latest Dorion release: {}", release.tag_name);

  // Find the release that ends with ".dmg", that's the MacOS release
  let mut release_name = String::new();

  for name in release.release_names {
    if name.ends_with(".msi") {
      release_name = name;
      break;
    }
  }

  let path = std::env::temp_dir();

  println!("Downloading {}...", release_name);

  let release_path = download_release(
    "SpikeHD",
    "Dorion",
    release.tag_name.clone(),
    release_name.clone(),
    path.clone(),
  );

  // Kill Dorion BEFORE we install
  println!("Attempting to kill Dorion process...");

  let mut cmd = std::process::Command::new("taskkill");
  cmd.arg("/F");
  cmd.arg("/IM");
  cmd.arg("Dorion.exe");

  cmd.spawn().unwrap().wait().unwrap();

  println!("Installing {:?}...", release_path.clone());

  // Install from the MSI in quiet mode
  let mut cmd = std::process::Command::new("msiexec");
  cmd.arg("/i");
  cmd.arg(
    release_path
      .into_os_string()
      .into_string()
      .unwrap()
  );

  println!("Running {:?}", cmd);

  cmd.spawn().unwrap();

  std::process::exit(0);
}

#[cfg(target_os = "windows")]
pub fn update_main_kinda() {
  // Same as the MSI, but we just download the zip file instead and open explorer to highlight it
  let release = get_release("SpikeHD", "Dorion");

  println!("Latest Dorion release: {}", release.tag_name);

  // Find the release that ends with ".zip", that should be the Windows release
  let mut release_name = String::new();

  for name in release.release_names {
    if name.ends_with(".zip") && name.contains("win64") {
      release_name = name;
      break;
    }
  }

  let path = std::env::temp_dir();

  println!("Downloading {}...", release_name);

  let release_path = download_release(
    "SpikeHD",
    "Dorion",
    release.tag_name.clone(),
    release_name.clone(),
    path.clone(),
  );

  println!("Opening {:?}...", release_path.clone());

  // Open the folder the zip is in and highlight
  let mut cmd = std::process::Command::new("explorer");
  cmd.arg("/select,");
  cmd.arg(
    release_path
      .into_os_string()
      .into_string()
      .unwrap()
  );

  cmd.spawn().unwrap();

  println!("Attempting to kill Dorion process...");

  // Also kill the main Dorion process if we can
  let mut cmd = std::process::Command::new("taskkill");
  cmd.arg("/F");
  cmd.arg("/IM");
  cmd.arg("Dorion.exe");

  cmd.spawn().unwrap().wait().unwrap();

  std::process::exit(0);
}

/**
 * Download the DMG and open
 */
#[cfg(target_os = "macos")]
pub fn update_main() {
  let release = get_release("SpikeHD", "Dorion");

  println!("Latest Dorion release: {}", release.tag_name);

  // Find the release that ends with ".dmg", that's the MacOS release
  let mut release_name = String::new();

  for name in release.release_names {
    if name.ends_with(".dmg") {
      release_name = name;
      break;
    }
  }

  let path = std::env::temp_dir();

  println!("Downloading {}...", release_name);

  let release_path = download_release(
    "SpikeHD",
    "Dorion",
    release.tag_name.clone(),
    release_name.clone(),
    path.clone(),
  );

  println!("Opening {:?}...", release_path.clone());

  // Open the mounted DMG
  let mut cmd = std::process::Command::new("open");
  cmd.arg(release_path);

  cmd.spawn().unwrap();

  println!("Attempting to kill Dorion process...");

  // Also kill the main Dorion process if we can
  let mut cmd = std::process::Command::new("pkill");
  cmd.arg("-9");
  cmd.arg("Dorion");

  cmd.spawn().unwrap();
}

/**
 * Do nothing, too hard to know where we were sourced from on Linux
 */
#[cfg(target_os = "linux")]
pub fn update_main() {}
