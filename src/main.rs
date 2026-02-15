use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

// --- ENUMS & STRUCTS ---

#[derive(Debug)]
enum UpdateEvent {
    Message(String),
    Success(String), // Version
    UpToDate,
    Error(String),
}

// --- MAIN EXECUTION ---

fn main() -> anyhow::Result<()> {
    // 1. Health Check (Run by the updater to verify the new binary)
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--list-ignored".to_string()) {
        let state = UpdateState::load();
        println!("Ignored versions: {:?}", state.ignored_versions);
        return Ok(());
    }

    if args.contains(&"--simulate-failure".to_string()) {
        println!("SIMULATED FAILURE: Exiting with error.");
        std::process::exit(1);
    }

    if args.contains(&"--test-blacklist".to_string()) {
        let mut state = UpdateState::load();
        state.mark_bad("9.9.9".to_string())?;
        println!("Marked 9.9.9 as bad.");
        return Ok(());
    }

    if args.contains(&"--health-check".to_string()) {
        // In a real app, check config integrity or basic startup logic here
        println!("Health check passed!");
        return Ok(());
    }

    println!("App Version: {}", env!("CARGO_PKG_VERSION"));
    
    // 2. Spawn the Update Thread
    let (tx, rx) = channel();
    thread::spawn(move || {
        if let Err(e) = run_background_update(tx.clone()) {
            let _ = tx.send(UpdateEvent::Error(e.to_string()));
        }
    });

    // 3. Main Application Loop (The "Animation")
    let spinner = vec!['|', '/', '-', '\\'];
    let mut idx = 0;
    let mut update_status = "Checking for updates in background...".to_string();

    // We run for a limited time for the demo, or until the user interrupts.
    // In a real app, this would be your main task.
    loop {
        // A. Process Update Events (Non-blocking)
        match rx.try_recv() {
            Ok(event) => match event {
                UpdateEvent::Message(msg) => update_status = msg,
                UpdateEvent::UpToDate => update_status = "System is up to date.".to_string(),
                UpdateEvent::Success(v) => update_status = format!("Update ready! Restart to use v{}", v),
                UpdateEvent::Error(e) => update_status = format!("Update failed: {}", e),
            },
            Err(TryRecvError::Empty) => {} // No message
            Err(TryRecvError::Disconnected) => {
                // Thread finished, we can choose to exit or continue
                // For this demo, we'll continue for a bit to show the final status
            }
        }

        // B. Render UI
        // Clear line and print status with padding to clear old messages
        print!("\r[{}] Application Running... Status: {:<50}", spinner[idx], update_status);
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        // C. Update Animation State
        idx = (idx + 1) % spinner.len();
        thread::sleep(Duration::from_millis(100));

        // D. Optional: Exit condition for demo
        // For demonstration purposes, we don't exit automatically.
        // The user would typically Ctrl+C after seeing the status.
    }
}

// --- UPDATE LOGIC (Runs in Background) ---

fn run_background_update(tx: Sender<UpdateEvent>) -> anyhow::Result<()> {
    let current_exe = env::current_exe()?;
    let backup_path = current_exe.with_extension("bak");
    let mut state = UpdateState::load();

    // 1. Configure Updater
    let os = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => std::env::consts::ARCH,
    };
    let target = format!("{}-{}", os, arch);
    
    // Embed public key (ensure zipsign.pub is in project root)
    let public_key: [u8; 32] = *include_bytes!("../zipsign.pub");

    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner("plops")
        .repo_name("rs-example-self-update")
        .bin_name("rs-example-self-update") // Important: Matches binary name inside archive
        .target(&target)
        .current_version(env!("CARGO_PKG_VERSION"))
        .verifying_keys(vec![public_key]);

    // 2. Check for Release (Peek)
    tx.send(UpdateEvent::Message("Querying GitHub...".into()))?;
    let release = builder.build()?.get_latest_release()?;

    // 3. Blacklist Check
    if state.is_bad(&release.version) {
        tx.send(UpdateEvent::Message(format!("Skipping bad version {}", release.version)))?;
        return Ok(());
    }

    if !self_update::version::bump_is_greater(env!("CARGO_PKG_VERSION"), &release.version)? {
        tx.send(UpdateEvent::UpToDate)?;
        return Ok(());
    }

    // 4. Update Sequence
    tx.send(UpdateEvent::Message(format!("Downloading v{}...", release.version)))?;

    // Create Backup
    fs::copy(&current_exe, &backup_path)?;

    // Perform Update (Swap binary on disk)
    // Note: On Windows, self_update renames the running file to allow writing the new one.
    // The running process continues in memory fine.
    match builder.build()?.update() {
        Ok(status) => {
            if !status.updated() {
                tx.send(UpdateEvent::UpToDate)?;
                return Ok(());
            }

            let new_version = status.version().to_string();
            tx.send(UpdateEvent::Message("Verifying new binary health...".into()))?;

            // 5. Health Check
            let output = Command::new(&current_exe)
                .arg("--simulate-failure") // SIMULATE FAILURE FOR TESTING
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    // Success! Clean backup
                    let _ = fs::remove_file(&backup_path);
                    tx.send(UpdateEvent::Success(new_version))?;
                }
                _ => {
                    // Fail! Rollback
                    tx.send(UpdateEvent::Message("Health check failed. Rolling back...".into()))?;
                    
                    // Mark bad
                    state.mark_bad(new_version.clone())?;
                    
                    // Restore backup
                    // On Windows, we overwrite the "new" broken file with the backup
                    fs::rename(&backup_path, &current_exe)?;
                    tx.send(UpdateEvent::Error(format!("Version {} broken. Rolled back.", new_version)))?;
                }
            }
        }
        Err(e) => {
            // Network/Signature error - restore backup just in case
            if backup_path.exists() {
                let _ = fs::rename(&backup_path, &current_exe);
            }
            return Err(e.into());
        }
    }

    Ok(())
}

// --- PERSISTENT STATE (The Blacklist) ---

#[derive(Serialize, Deserialize, Default)]
struct UpdateState {
    ignored_versions: HashSet<String>,
}

impl UpdateState {
    fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(file) = std::fs::File::open(path) {
                if let Ok(state) = serde_json::from_reader(file) {
                    return state;
                }
            }
        }
        Self::default()
    }

    fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn mark_bad(&mut self, version: String) -> anyhow::Result<()> {
        self.ignored_versions.insert(version);
        self.save()
    }

    fn is_bad(&self, version: &str) -> bool {
        let v_clean = version.trim_start_matches('v');
        self.ignored_versions.contains(v_clean) || self.ignored_versions.contains(version)
    }

    fn get_path() -> PathBuf {
        // Cross-platform cache location
        if let Some(proj) = ProjectDirs::from("com", "plops", "rs-example-self-update") {
            proj.cache_dir().join("state.json")
        } else {
            PathBuf::from("update_state.json")
        }
    }
}