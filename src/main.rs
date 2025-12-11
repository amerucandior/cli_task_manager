use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::path::PathBuf;

mod task;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add { description: String },
    /// List tasks (use --all to include completed)
    List {
        #[arg(short, long)]
        all: bool,
    },
    /// Mark a task as completed
    Done { id: u32 },
    /// Remove a task
    Remove { id: u32 },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data_path = get_data_path()?;
    let mut tasks = task::load_tasks(&data_path)?;

    match cli.command {
        Commands::Add { description } => {
            task::add_task(&mut tasks, description)?;
            task::save_tasks(&data_path, &tasks)?;
        }
        Commands::List { all } => task::list_tasks(&tasks, all),
        Commands::Done { id } => {
            task::mark_done(&mut tasks, id)?;
            task::save_tasks(&data_path, &tasks)?;
        }
        Commands::Remove { id } => {
            task::remove_task(&mut tasks, id)?;
            task::save_tasks(&data_path, &tasks)?;
        }
    }
    Ok(())
}

fn get_data_path() -> anyhow::Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("ian", "mwirigi", "cli_task_manager")
        .ok_or_else(|| anyhow::anyhow!("Unable to determine data directory"))?;
    Ok(proj_dirs.data_local_dir().join("tasks.json"))
}
