use crate::models::{ChildProcessMap, Instance, InstanceEngine, InstanceSettings, InstanceState};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn start_instance(
    app: AppHandle,
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");
    let mut instance_path = PathBuf::new();
    let mut settings = InstanceSettings::default();

    // Find instance
    if instances_dir.exists() {
        for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                if let Ok(content) = fs::read_to_string(&json_path) {
                    if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                        if inst.id == id {
                            instance_path = entry.path();
                            settings = inst.settings;
                            break;
                        }
                    }
                }
            }
        }
    }

    if !instance_path.exists() {
        return Err("Instance not found".to_string());
    }

    let server_jar = instance_path.join(".minecraft").join(&settings.jar_file);
    if !server_jar.exists() {
        return Err(format!("Server JAR '{}' not found", settings.jar_file));
    }

    // Java Command
    let mut cmd = Command::new("java");
    cmd.current_dir(instance_path.join(".minecraft"));

    cmd.arg(format!("-Xms{}M", settings.min_ram));
    cmd.arg(format!("-Xmx{}M", settings.max_ram));
    cmd.arg("-Dfile.encoding=UTF-8");

    if !settings.args.is_empty() {
        for arg in settings.args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    cmd.arg("-jar");
    cmd.arg(&settings.jar_file);
    cmd.arg("nogui");

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    // Handle Stdout
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let id_clone = id.clone();
        let state_clone = state.0.clone();
        let instances_dir_clone = instances_dir.clone();

        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = app_clone.emit("server-log", (id_clone.clone(), l));
                }
            }
            // Process exit
            if let Ok(mut map) = state_clone.lock() {
                map.remove(&id_clone);
            }
            let _ = update_instance_state(&instances_dir_clone, &id_clone, InstanceState::Stopped);
            let _ = app_clone.emit("instance-update", ());
        });
    }

    // Handle Stderr
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let id_clone = id.clone();
        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = app_clone.emit("server-log", (id_clone.clone(), l));
                }
            }
        });
    }

    // Save Child
    state
        .0
        .lock()
        .map_err(|_| "Lock failed")?
        .insert(id.clone(), child);

    // Update State
    update_instance_state(&instances_dir, &id, InstanceState::Running)?;
    app.emit("instance-update", ()).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn stop_instance(
    app: AppHandle,
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
    let mut stop_cmd = "stop";

    // Determine stop command (check for proxy)
    // Simplified check: if velocity/waterfall, use "end"
    if let Ok(app_data) = app.path().app_data_dir() {
        let instances_dir = app_data.join("instances");
        if instances_dir.exists() {
            for entry in fs::read_dir(instances_dir).into_iter().flatten().flatten() {
                let json_path = entry.path().join("instance.json");
                if json_path.exists() {
                    if let Ok(content) = fs::read_to_string(json_path) {
                        if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                            if inst.id == id {
                                match inst.loader {
                                    InstanceEngine::Velocity | InstanceEngine::Waterfall => {
                                        stop_cmd = "end"
                                    }
                                    _ => {}
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(child) = map.get_mut(&id) {
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "{}", stop_cmd).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn force_kill_instance(
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    // Note: command name changed from kill_instance to force_kill_instance to match frontend expectations if necessary
    // But frontend calls "kill_instance". I should probably keep it "kill_instance" or update frontend.
    // Keeping it "kill_instance" in Export, but function name can be anything.
    // Wait, the tauri command name IS the function name unless specified.
    // I will rename this to kill_instance to match lib.rs original.
    let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
    if let Some(child) = map.get_mut(&id) {
        let _ = child.kill();
        map.remove(&id);
    }
    Ok(())
}

// Rename for export correctness
#[tauri::command]
pub async fn kill_instance(state: State<'_, ChildProcessMap>, id: String) -> Result<(), String> {
    force_kill_instance(state, id).await
}

#[tauri::command]
pub async fn send_command(
    state: State<'_, ChildProcessMap>,
    id: String,
    command: String,
) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|_| "Lock failed")?;
    if let Some(child) = map.get_mut(&id) {
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "{}", command).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// Private Helper
fn update_instance_state(
    instances_dir: &std::path::Path,
    id: &str,
    new_state: InstanceState,
) -> Result<(), String> {
    for entry in fs::read_dir(instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            // Using a resilient deserialize or just default
            if let Ok(mut instance) = serde_json::from_str::<Instance>(&content) {
                if instance.id == id {
                    instance.state = new_state;
                    let new_json =
                        serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
                    fs::write(json_path, new_json).map_err(|e| e.to_string())?;
                    break;
                }
            }
        }
    }
    Ok(())
}
