use std::{
    fs,
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{storage::paths, workspace::model::WorkspaceState};

const WORKSPACE_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
struct PersistedWorkspace {
    version: u32,
    #[serde(flatten)]
    state: WorkspaceState,
}

pub fn load_or_default() -> WorkspaceState {
    match load_workspace() {
        Ok(Some(workspace)) => workspace,
        Ok(None) => WorkspaceState::mock(),
        Err(error) => {
            eprintln!(
                "failed to load workspace from {}: {error}",
                paths::workspace_file().display()
            );
            WorkspaceState::mock()
        }
    }
}

pub fn load_workspace() -> io::Result<Option<WorkspaceState>> {
    let path = paths::workspace_file();

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)?;
    let persisted: PersistedWorkspace = serde_json::from_str(&contents).map_err(invalid_data)?;

    if persisted.version != WORKSPACE_VERSION {
        return Err(invalid_data(format!(
            "unsupported workspace version {}; expected {WORKSPACE_VERSION}",
            persisted.version
        )));
    }

    Ok(Some(persisted.state))
}

pub fn save_workspace(workspace: &WorkspaceState) -> io::Result<()> {
    let path = paths::workspace_file();
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("workspace path has no parent: {}", path.display()),
        )
    })?;

    fs::create_dir_all(parent)?;

    let persisted = PersistedWorkspace {
        version: WORKSPACE_VERSION,
        state: workspace.clone(),
    };
    let json = serde_json::to_string_pretty(&persisted).map_err(invalid_data)?;

    atomic_write(&path, json.as_bytes())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut last_error = None;

    for _ in 0..10 {
        let temp_path = path.with_extension(format!("json.tmp.{:016x}", rand::random::<u64>()));

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(mut temp_file) => {
                if let Err(error) = temp_file.write_all(bytes) {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }

                if let Err(error) = temp_file.sync_all() {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }

                drop(temp_file);

                return fs::rename(temp_path, path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "failed to create unique temporary workspace file",
        )
    }))
}

fn invalid_data(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
