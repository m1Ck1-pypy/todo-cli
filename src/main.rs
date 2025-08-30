use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use dirs_next::data_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct TodoTask {
    id: Uuid,
    index: u32,
    title: String,
    is_complited: bool,
    created_at: DateTime<Utc>,
}

#[derive(Parser)]
#[command(name = "Todo CLI")]
#[command(about = "Простой менеджер задач", version = "0.1")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Добавить новую задачу
    Add {
        #[arg(help = "Текст задачи")]
        title: String,
    },
    /// Показать все задачи
    List,
    /// Отметить задачу как "выполнена"
    Done {
        #[arg(help = "Index задачи")]
        index: u32,
    },
    /// Удаление задачи
    Remove {
        #[arg(help = "Index задачи")]
        index: u32,
    },
    /// Очистить файл
    Clear,
}




fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = data_file_path(); // или создать локально файл рядом с бинарником
    let mut tasks = load_tasks(&path)?;

    match &cli.command {
        Commands::Add { title } => {
            let new_task_index = tasks.iter().map(|t| t.index).max().unwrap_or(0) + 1;
            let now = Utc::now();
            tasks.push(TodoTask {
                id: Uuid::new_v4(),
                index: new_task_index,
                title: title.clone(),
                is_complited: false,
                created_at: now,
            });
            println!("✅ Задача '{title}' добавлена");
        }
        Commands::List => {
            if tasks.is_empty() {
                println!("📝 Список задач пуст.");
            } else {
                println!("📋 Список задач:");
                for task in &tasks {
                    let status = if task.is_complited { "✅" } else { "❌" };
                    let created_local: DateTime<Local> = task.created_at.into();
                    let created_str = created_local.format("%H:%M, %d.%m.%Y").to_string();

                    println!("{}. [{status}] {} - {created_str}", task.index, task.title);
                }
            }
        }
        Commands::Done { index } => {
            if let Some(task) = tasks.iter_mut().find(|t| t.index == *index) {
                task.is_complited = true;
                println!("🎉 Задача '{}' выполнена", { &task.title });
            } else {
                anyhow::bail!("❌ Задача с индексом {index} не найдена.");
            }
        }
        Commands::Remove { index } => {
            let len_before = tasks.len();
            tasks.retain(|t| t.index != *index);
            println!("{tasks:?}");
            if tasks.len() < len_before {
                println!("🗑️ Задача с индексом {index} удалена.");

                tasks.sort_by_key(|t| t.index);
                for (index, task) in tasks.iter_mut().enumerate() {
                    task.index = (index + 1) as u32;
                }
                println!(
                    "🔄 ID пересчитаны: теперь задачи пронумерованы от 1 до {}.",
                    tasks.len()
                );
            } else {
                anyhow::bail!("❌ Задача с индексом {index} не найдена.");
            }
        }
        Commands::Clear => {
            tasks.clear();
            println!("🗑️ Файл очищен.");
        }
    }

    save_tasks(&path, &tasks)?;
    Ok(())
}

fn load_tasks(path: &PathBuf) -> Result<Vec<TodoTask>, anyhow::Error> {
    let data = fs::read_to_string(path)?;

    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    let tasks = serde_json::from_str(trimmed)?;
    Ok(tasks)
}

fn save_tasks(path: &PathBuf, tasks: &[TodoTask]) -> Result<(), anyhow::Error> {
    let data = serde_json::to_string_pretty(tasks)?;
    fs::write(path, data)?; // или создать локально файл рядом с бинарником
    Ok(())
}

fn data_file_path() -> PathBuf {
    let mut path = data_dir().expect("Не удалось найти папку данных");
    path.push("todo-cli"); // Папка приложения
    fs::create_dir_all(&path).expect("Не удалось создать папку данных");
    path.push("todos.json"); // Файл задач
    // C:\Users\{user}\AppData\Roaming\todo-cli\todos.json
    path
}
