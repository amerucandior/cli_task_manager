use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub completed: bool,
}

pub fn load_tasks(path: &Path) -> anyhow::Result<Vec<Task>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read tasks file at {}", path.display()))?;

    if data.trim().is_empty() {
        return Ok(Vec::new());
    }

    let tasks = serde_json::from_str(&data).with_context(|| {
        format!(
            "Failed to parse tasks file at {}. Ensure it contains valid JSON.",
            path.display()
        )
    })?;
    Ok(tasks)
}

pub fn save_tasks(path: &Path, tasks: &[Task]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create data directory at {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(tasks).context("Failed to serialize tasks to JSON")?;

    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path).with_context(|| {
            format!(
                "Failed to create temporary tasks file at {}",
                tmp_path.display()
            )
        })?;
        file.write_all(data.as_bytes())
            .with_context(|| format!("Failed to write tasks to {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("Failed to flush tasks to {}", tmp_path.display()))?;
    }

    fs::rename(&tmp_path, path)
        .map_err(|err| {
            let _ = fs::remove_file(&tmp_path);
            err
        })
        .with_context(|| {
            format!(
                "Failed to replace {} with {}",
                path.display(),
                tmp_path.display()
            )
        })?;
    Ok(())
}

pub fn add_task(tasks: &mut Vec<Task>, description: String) -> anyhow::Result<()> {
    let description = description.trim();
    if description.is_empty() {
        bail!("Task description cannot be empty");
    }

    let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    tasks.push(Task {
        id: next_id,
        description: description.to_owned(),
        completed: false,
    });
    Ok(())
}

pub fn list_tasks(tasks: &[Task], all: bool) {
    let mut shown = false;
    for task in tasks.iter().filter(|t| all || !t.completed) {
        let status = if task.completed { "[x]" } else { "[ ]" };
        println!("{} {}: {}", status, task.id, task.description);
        shown = true;
    }

    if !shown {
        if tasks.is_empty() {
            println!("No tasks found.");
        } else {
            println!("No tasks to show (use --all to include completed).");
        }
    }
}

pub fn mark_done(tasks: &mut Vec<Task>, id: u32) -> anyhow::Result<()> {
    match tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.completed = true;
            Ok(())
        }
        None => bail!("No task with id {}", id),
    }
}

pub fn remove_task(tasks: &mut Vec<Task>, id: u32) -> anyhow::Result<()> {
    let len_before = tasks.len();
    tasks.retain(|t| t.id != id);
    if tasks.len() < len_before {
        Ok(())
    } else {
        bail!("No task with id {}", id)
    }
}
