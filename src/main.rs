// PASTEL TODO — Hard Layout 60x30 (Tab Header ×2, Tab Body ×1)
// -------------------------------------------------------------
// Fixed layout: width 60, height 30. ASCII borders only.
// Header uses double tab (\t\t), body uses single tab (\t).
// If terminal smaller than 60x30 → exit with error.
// -------------------------------------------------------------

use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const ACCENT: &str = "\x1b[38;5;219m";
const TODO_COLOR: &str = "\x1b[38;5;153m";
const DONE_COLOR: &str = "\x1b[38;5;151m";
const DATE_COLOR: &str = "\x1b[38;5;223m";
const FOLDER_COLOR: &str = "\x1b[38;5;212m";
const TIP_TEXT: &str = "\x1b[38;5;251m";
const HEADER_BG_INVERT: &str = "\x1b[48;5;60m";
const HEADER_FG_INVERT: &str = "\x1b[38;5;218m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const DIM: &str = "\x1b[2m";
const VALUE_COLOR: &str = "\x1b[38;5;159m";
const SUMMARY_COLOR: &str = "\x1b[38;5;183m";
const POINTER_COLOR: &str = "\x1b[38;5;218m";
const BORDER_COLOR: &str = "\x1b[38;5;225m";

const TABLE_WIDTH: usize = 60;
const TASK_COLUMN_WIDTH: usize = 36;
const MAX_VISIBLE_TASKS: usize = 7;

// 👉 Layout size configuration
// -------------------------------------------------------------
// Minimum terminal size (change these numbers if needed)
const MIN_WIDTH: u16 = 60;
// Minimum height (change if you want taller layout)
const MIN_HEIGHT: u16 = 30;
// -------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Task {
    text: String,
    done: bool,
    folder: String,
    created_at: DateTime<Local>,
}

enum Mode {
    Command,
    CommandInput(CommandContext),
    Navigate { selected: usize },
}

enum CommandContext {
    Add { buffer: String },
    Folder { buffer: String },
    Delete { buffer: String },
}

enum CommandAction {
    AddTask(String),
    SwitchFolder(String),
    DeleteTask(usize),
    DeleteFolder(String),
}

struct RawModeGuard;
impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn main() {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    if cols < MIN_WIDTH || rows < MIN_HEIGHT {
        eprintln!(
            "\x1b[31mError: Terminal must be at least {}x{} (current: {}x{})\x1b[0m",
            MIN_WIDTH, MIN_HEIGHT, cols, rows
        );
        std::process::exit(1);
    }

    let mut tasks = load_tasks();
    let mut current_folder = tasks
        .first()
        .map(|t| t.folder.clone())
        .unwrap_or_else(|| "inbox".to_string());

    let _raw = RawModeGuard::new().expect("Cannot enable raw mode");

    let mut mode = Mode::Command;
    let mut buffer = String::new();

    loop {
        render(&tasks, &current_folder, &mode, &buffer);

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if handle_key(key, &mut mode, &mut buffer, &mut tasks, &mut current_folder) {
                    break;
                }
            }
        }
    }
}

// ---- Render fixed layout 60x30 ----
fn render(tasks: &[Task], folder: &str, mode: &Mode, cmd: &str) {
    print!("\x1b[2J\x1b[H"); // clear screen + move to top-left

    let folder_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.folder == folder)
        .map(|(idx, _)| idx)
        .collect();
    let folder_count = folder_indices.len();
    let selected_idx = match mode {
        Mode::Navigate { selected } if folder_count > 0 => Some((*selected).min(folder_count - 1)),
        _ => None,
    };
    let mut start = 0usize;
    if let Some(sel) = selected_idx {
        if sel + 1 > MAX_VISIBLE_TASKS {
            start = sel + 1 - MAX_VISIBLE_TASKS;
        }
        if folder_count > MAX_VISIBLE_TASKS && start + MAX_VISIBLE_TASKS > folder_count {
            start = folder_count - MAX_VISIBLE_TASKS;
        }
    } else if folder_count > MAX_VISIBLE_TASKS {
        start = folder_count - MAX_VISIBLE_TASKS;
    }

    let horizontal = "─".repeat(TABLE_WIDTH);
    println!(
        "\r{border}╭{line}╮{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );
    table_row(&format!(
        "{HEADER_BG_INVERT}{HEADER_FG_INVERT}{BOLD} PASTEL TODO {RESET}"
    ));
    println!(
        "\r{border}├{line}┤{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );

    table_row(&format!(
        "{label}Total:{reset} {value}{total:<3}{reset}  {label}Folder:{reset} {folder_color}{folder}{reset} ({value}{count}{reset})",
        label = ACCENT,
        reset = RESET,
        value = VALUE_COLOR,
        total = tasks.len(),
        folder_color = FOLDER_COLOR,
        folder = folder,
        count = folder_count
    ));
    table_row(&format!(
        "{label}Folder Name:{reset} {folder_color}{folder}{reset}",
        label = ACCENT,
        reset = RESET,
        folder_color = FOLDER_COLOR,
        folder = folder
    ));
    println!(
        "\r{border}├{line}┤{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );

    table_row(&format!(
        "{label} No.  ○  Task                                    Date{reset}",
        label = ACCENT,
        reset = RESET
    ));
    println!(
        "\r{border}├{line}┤{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );

    let visible_items: Vec<(usize, &Task)> = folder_indices
        .iter()
        .enumerate()
        .skip(start)
        .take(MAX_VISIBLE_TASKS)
        .map(|(order, idx)| (order, &tasks[*idx]))
        .collect();

    for (order, task) in &visible_items {
        let is_selected = selected_idx == Some(*order);
        let pointer = if is_selected {
            format!("{POINTER_COLOR}›{RESET}")
        } else {
            format!("{BORDER_COLOR}•{RESET}")
        };
        let status = if task.done {
            format!("{DONE_COLOR}✓{RESET}")
        } else {
            format!("{TODO_COLOR}○{RESET}")
        };
        let task_label = truncate(&task.text, TASK_COLUMN_WIDTH);
        let padded_label = format!("{:<width$}", task_label, width = TASK_COLUMN_WIDTH);
        let task_colored = if task.done {
            format!("{DONE_COLOR}{}{RESET}", padded_label)
        } else {
            format!("{TODO_COLOR}{}{RESET}", padded_label)
        };
        let date = format!("{DATE_COLOR}{}{RESET}", task.created_at.format("%d/%m/%y"));
        let number = format!("{VALUE_COLOR}{:>2}{RESET}", order + 1);
        let row = format!(
            "{pointer} {number}.  {status}  {task} {date}",
            pointer = pointer,
            number = number,
            status = status,
            task = task_colored,
            date = date
        );
        table_row(&row);
    }

    for _ in visible_items.len()..MAX_VISIBLE_TASKS {
        table_row("");
    }

    println!(
        "\r{border}├{line}┤{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );
    let summary_plain = if folder_count == 0 {
        " Showing 0 tasks in this folder.".to_string()
    } else {
        let start_display = start + 1;
        let end_display = start + visible_items.len();
        format!(
            " Showing {}-{} of {} tasks in this folder.",
            start_display, end_display, folder_count
        )
    };
    let summary = format!("{SUMMARY_COLOR}{summary_plain}{RESET}");
    table_row(&summary);
    println!(
        "\r{border}├{line}┤{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );

    match mode {
        Mode::Command => {
            let label = " command: ";
            let available = TABLE_WIDTH.saturating_sub(label.len());
            let display = if cmd.is_empty() {
                format!("{DIM}(type a command and press Enter){RESET}")
            } else {
                let clipped = truncate(cmd, available);
                clipped
            };
            table_row(&format!("{label}{display}"));
            table_row("");
        }
        Mode::CommandInput(context) => match context {
            CommandContext::Add { buffer } => {
                table_row(" command: add");
                let label = " add: ";
                let available = TABLE_WIDTH.saturating_sub(label.len());
                let display = if buffer.is_empty() {
                    format!("{DIM}(describe the task, Enter to save){RESET}")
                } else {
                    let clipped = truncate(buffer, available);
                    clipped
                };
                table_row(&format!("{label}{display}"));
            }
            CommandContext::Folder { buffer } => {
                table_row(" command: folder");
                let label = " folder: ";
                let available = TABLE_WIDTH.saturating_sub(label.len());
                let display = if buffer.is_empty() {
                    format!("{DIM}(type folder name, Enter to switch){RESET}")
                } else {
                    let clipped = truncate(buffer, available);
                    clipped
                };
                table_row(&format!("{label}{display}"));
            }
            CommandContext::Delete { buffer } => {
                table_row(" command: delete");
                let label = " delete: ";
                let available = TABLE_WIDTH.saturating_sub(label.len());
                let display = if buffer.is_empty() {
                    format!("{DIM}(number or 'folder name'){RESET}")
                } else {
                    let clipped = truncate(buffer, available);
                    clipped
                };
                table_row(&format!("{label}{display}"));
            }
        },
        Mode::Navigate { .. } => {
            table_row(&format!(
                "{ACCENT} navigate:{RESET} {VALUE_COLOR}↑/↓ move{RESET}, {VALUE_COLOR}d marks done{RESET}, {VALUE_COLOR}Esc exits{RESET}"
            ));
            table_row("");
        }
    }

    let tip_variants = [
        "Tip: Tap add for a quick idea, folder to regroup, delete to tidy up.",
        "Tip: Folder keeps contexts neat; add logs tasks; delete clears the clutter.",
        "Tip: Need a reset? add captures, folder jumps, delete prunes.",
    ];
    let tip_tick = (Local::now().timestamp() / 15).max(0) as u64;
    let hash = tip_tick
        .wrapping_mul(636_413_622_384_679_3005)
        .rotate_left(7);
    let tip_index = (hash % tip_variants.len() as u64) as usize;
    let tip_line = format!("{TIP_TEXT}{DIM}{ITALIC}{}{RESET}", tip_variants[tip_index]);
    table_row(&tip_line);

    println!(
        "\r{border}╰{line}╯{reset}",
        border = BORDER_COLOR,
        line = &horizontal,
        reset = RESET
    );
    let _ = io::stdout().flush();
}

fn truncate(text: &str, len: usize) -> String {
    if text.chars().count() <= len {
        return text.to_string();
    }
    if len <= 3 {
        return ".".repeat(len);
    }

    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if i >= len - 3 {
            out.push_str("...");
            break;
        }
        out.push(c);
    }
    out
}

fn clamp_display(content: &str, width: usize) -> (String, usize) {
    let mut buf = String::with_capacity(content.len());
    let mut visible = 0usize;
    let mut chars = content.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            buf.push(ch);
            while let Some(next) = chars.next() {
                buf.push(next);
                if next == 'm' {
                    break;
                }
            }
            continue;
        }

        if visible >= width {
            continue;
        }

        buf.push(ch);
        visible += 1;
    }
    (buf, visible.min(width))
}

fn table_row(content: &str) {
    let (prepared, visible) = clamp_display(content, TABLE_WIDTH);
    let padding = TABLE_WIDTH.saturating_sub(visible);
    let padding_str = " ".repeat(padding);
    println!(
        "\r{border}│{reset}{prepared}{padding}{border}│{reset}",
        border = BORDER_COLOR,
        reset = RESET,
        prepared = prepared,
        padding = padding_str
    );
}

fn handle_key(
    key: KeyEvent,
    mode: &mut Mode,
    buffer: &mut String,
    tasks: &mut Vec<Task>,
    folder: &mut String,
) -> bool {
    match mode {
        Mode::Command => handle_command(key, mode, buffer, tasks, folder),
        Mode::CommandInput(_) => handle_command_input(key, mode, buffer, tasks, folder),
        Mode::Navigate { .. } => {
            handle_navigate(key, mode, tasks, folder);
            false
        }
    }
}

fn handle_command(
    key: KeyEvent,
    mode: &mut Mode,
    buf: &mut String,
    tasks: &mut Vec<Task>,
    folder: &mut String,
) -> bool {
    match key.code {
        KeyCode::Esc => buf.clear(),
        KeyCode::Backspace => {
            buf.pop();
        }
        KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            let folder_len = tasks.iter().filter(|t| t.folder == *folder).count();
            if folder_len > 0 {
                *mode = Mode::Navigate {
                    selected: folder_len - 1,
                };
            }
        }
        KeyCode::Char('q') => {
            println!("{ACCENT}See you later!{RESET}");
            println!();
            return true;
        }
        KeyCode::Enter => {
            let cmd = buf.trim().to_lowercase();
            if cmd == "add" {
                *mode = Mode::CommandInput(CommandContext::Add {
                    buffer: String::new(),
                });
            } else if cmd == "delete" {
                *mode = Mode::CommandInput(CommandContext::Delete {
                    buffer: String::new(),
                });
            } else if cmd == "folder" {
                *mode = Mode::CommandInput(CommandContext::Folder {
                    buffer: String::new(),
                });
            }
            buf.clear();
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                buf.push(c);
            }
        }
        _ => {}
    }
    false
}

fn handle_command_input(
    key: KeyEvent,
    mode: &mut Mode,
    cmd_buf: &mut String,
    tasks: &mut Vec<Task>,
    folder: &mut String,
) -> bool {
    let mut exit_to_command = false;
    let mut action: Option<CommandAction> = None;

    match mode {
        Mode::CommandInput(CommandContext::Add { buffer }) => match key.code {
            KeyCode::Esc => {
                buffer.clear();
                exit_to_command = true;
            }
            KeyCode::Char('b') if key.modifiers.is_empty() && buffer.is_empty() => {
                exit_to_command = true;
            }
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Enter => {
                let text = buffer.trim();
                if !text.is_empty() {
                    action = Some(CommandAction::AddTask(text.to_string()));
                    buffer.clear();
                    exit_to_command = true;
                }
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    buffer.push(c);
                }
            }
            _ => {}
        },
        Mode::CommandInput(CommandContext::Folder { buffer }) => match key.code {
            KeyCode::Esc => {
                buffer.clear();
                exit_to_command = true;
            }
            KeyCode::Char('b') if key.modifiers.is_empty() && buffer.is_empty() => {
                exit_to_command = true;
            }
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Enter => {
                let name = buffer.trim();
                if !name.is_empty() {
                    action = Some(CommandAction::SwitchFolder(name.to_string()));
                    buffer.clear();
                    exit_to_command = true;
                }
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    buffer.push(c);
                }
            }
            _ => {}
        },
        Mode::CommandInput(CommandContext::Delete { buffer }) => match key.code {
            KeyCode::Esc => {
                buffer.clear();
                exit_to_command = true;
            }
            KeyCode::Char('b') if key.modifiers.is_empty() && buffer.is_empty() => {
                exit_to_command = true;
            }
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Enter => {
                let input = buffer.trim();
                if !input.is_empty() {
                    let mut parts = input.split_whitespace();
                    let head = parts.next().unwrap_or("");
                    if head.eq_ignore_ascii_case("folder") {
                        let target = parts.collect::<Vec<_>>().join(" ");
                        let target = if target.is_empty() {
                            folder.clone()
                        } else {
                            target
                        };
                        action = Some(CommandAction::DeleteFolder(target));
                        buffer.clear();
                        exit_to_command = true;
                    } else if let Ok(idx) = input.parse::<usize>() {
                        if idx > 0 {
                            action = Some(CommandAction::DeleteTask(idx));
                            buffer.clear();
                            exit_to_command = true;
                        }
                    }
                }
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    buffer.push(c);
                }
            }
            _ => {}
        },
        _ => {}
    }

    if let Some(action) = action {
        match action {
            CommandAction::AddTask(text) => {
                tasks.push(Task {
                    text,
                    done: false,
                    folder: folder.clone(),
                    created_at: Local::now(),
                });
                save_tasks(tasks).ok();
            }
            CommandAction::SwitchFolder(name) => {
                *folder = name;
            }
            CommandAction::DeleteTask(number) => {
                let folder_tasks: Vec<_> = tasks
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.folder == *folder)
                    .collect();
                if let Some((real_idx, _)) = folder_tasks.get(number.saturating_sub(1)) {
                    tasks.remove(*real_idx);
                    save_tasks(tasks).ok();
                }
            }
            CommandAction::DeleteFolder(name) => {
                let current_name = if name.is_empty() {
                    folder.clone()
                } else {
                    name
                };
                let original_len = tasks.len();
                tasks.retain(|t| !t.folder.eq_ignore_ascii_case(&current_name));
                if tasks.len() != original_len {
                    save_tasks(tasks).ok();
                }
                if folder.eq_ignore_ascii_case(&current_name) {
                    if let Some(next) = tasks.first() {
                        *folder = next.folder.clone();
                    } else {
                        *folder = "inbox".to_string();
                    }
                }
            }
        }
    }

    if exit_to_command {
        *mode = Mode::Command;
        cmd_buf.clear();
    }

    false
}

fn handle_navigate(key: KeyEvent, mode: &mut Mode, tasks: &mut Vec<Task>, folder: &str) {
    let Mode::Navigate { selected } = mode else {
        return;
    };

    let folder_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.folder == folder)
        .map(|(idx, _)| idx)
        .collect();

    if folder_indices.is_empty() {
        *mode = Mode::Command;
        return;
    }

    let max_index = folder_indices.len() - 1;
    if *selected > max_index {
        *selected = max_index;
    }

    match key.code {
        KeyCode::Esc => *mode = Mode::Command,
        KeyCode::Up => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        KeyCode::Down => {
            if *selected < max_index {
                *selected += 1;
            }
        }
        KeyCode::Char('d') => {
            if let Some(&task_idx) = folder_indices.get(*selected) {
                if let Some(task) = tasks.get_mut(task_idx) {
                    task.done = true;
                    save_tasks(tasks).ok();
                }
            }
        }
        _ => {}
    }
}

// ---- File handling ----
fn config_path() -> PathBuf {
    let mut dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("pastel_todo");
    let _ = fs::create_dir_all(&dir);
    dir.push("tasks.tsv");
    dir
}

fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    let mut f = File::create(config_path())?;
    for t in tasks {
        let flag = if t.done { "1" } else { "0" };
        writeln!(
            f,
            "{}\t{}\t{}\t{}",
            flag,
            t.folder,
            t.created_at.to_rfc3339(),
            t.text
        )?;
    }
    Ok(())
}

fn load_tasks() -> Vec<Task> {
    let path = config_path();
    if !path.exists() {
        return Vec::new();
    }
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .flatten()
        .filter_map(|l| {
            let p: Vec<&str> = l.splitn(4, '\t').collect();
            if p.len() == 4 {
                let done = p[0] == "1";
                let folder = p[1].to_string();
                let created_at = DateTime::parse_from_rfc3339(p[2])
                    .ok()
                    .map(|d| d.with_timezone(&Local))
                    .unwrap_or_else(Local::now);
                let text = p[3].to_string();
                Some(Task {
                    text,
                    done,
                    folder,
                    created_at,
                })
            } else {
                None
            }
        })
        .collect()
}
