# 🎨 PASTEL TODO

A beautiful, minimalist terminal-based todo list manager written in Rust with pastel color aesthetics.

![Version](https://img.shields.io/badge/version-0.2.0-pink)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## ✨ Features

- 🎨 **Beautiful Pastel UI** - Eye-pleasing color scheme with ANSI terminal colors
- 📁 **Folder Organization** - Organize tasks into different folders/categories
- ⚡ **Fast & Lightweight** - Built with Rust for maximum performance
- 💾 **Persistent Storage** - Tasks automatically saved to disk
- ⌨️ **Vim-like Keybindings** - Efficient keyboard-driven navigation
- 📅 **Task Timestamps** - Automatic creation date tracking
- ✅ **Task Completion** - Mark tasks as done/undone
- 🖥️ **Fixed Layout** - Clean 60x30 terminal interface with ASCII borders

## 📋 Requirements

- **Terminal**: Minimum 60x30 characters (will exit with error if smaller)
- **Rust**: 1.56 or higher (2021 edition)
- **OS**: Linux, macOS, BSD, or any Unix-like system

## 🚀 Installation

### Option 1: Install from Source (Any Linux Distribution)

#### Step 1: Install Rust

If you don't have Rust installed, install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Step 2: Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/PASTELtodo.git
cd PASTELtodo

# Build in release mode
cargo build --release

# The binary will be at: target/release/pastel_todo
```

#### Step 3: Install System-wide (Optional)

```bash
# Copy to a location in your PATH
sudo cp target/release/pastel_todo /usr/local/bin/

# Or for user-only installation
cp target/release/pastel_todo ~/.local/bin/

# Make sure it's executable
chmod +x /usr/local/bin/pastel_todo  # or ~/.local/bin/pastel_todo
```

### Option 2: Direct Cargo Install

```bash
# Install directly with cargo
cargo install --path .
```

This will install the binary to `~/.cargo/bin/pastel_todo` (make sure `~/.cargo/bin` is in your PATH).

### Option 3: Distribution-Specific Package Managers

#### Arch Linux / Manjaro

```bash
# Using an AUR helper (e.g., yay)
# Note: Package must be created and published to AUR first
yay -S pastel-todo
```

#### Debian / Ubuntu

```bash
# Download the binary and install
wget https://github.com/yourusername/PASTELtodo/releases/latest/download/pastel_todo
chmod +x pastel_todo
sudo mv pastel_todo /usr/local/bin/
```

#### Fedora / RHEL / CentOS

```bash
# Download the binary and install
curl -LO https://github.com/yourusername/PASTELtodo/releases/latest/download/pastel_todo
chmod +x pastel_todo
sudo mv pastel_todo /usr/local/bin/
```

#### NixOS

Add to your `configuration.nix` or use nix-shell:

```bash
nix-shell -p rustc cargo
git clone https://github.com/yourusername/PASTELtodo.git
cd PASTELtodo
cargo build --release
```

### Option 4: macOS

```bash
# Install Rust via Homebrew (if not already installed)
brew install rust

# Clone and build
git clone https://github.com/yourusername/PASTELtodo.git
cd PASTELtodo
cargo build --release
sudo cp target/release/pastel_todo /usr/local/bin/
```

## 📖 Usage

### Starting the Application

Simply run:

```bash
pastel_todo
```

**Note**: Your terminal must be at least 60 columns wide and 30 rows tall, or the application will exit with an error message.

### Keyboard Controls

#### Command Mode (Default)

| Key | Action |
|-----|--------|
| `a` | Add a new task |
| `f` | Switch/create folder |
| `d` | Delete a task |
| `D` | Delete current folder |
| `n` | Enter navigation mode |
| `q` | Quit the application |

#### Navigation Mode

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `Space` | Toggle task completion (done/undone) |
| `Esc` | Return to command mode |

#### Input Mode

| Key | Action |
|-----|--------|
| `Enter` | Confirm input |
| `Esc` | Cancel and return to command mode |
| `Backspace` | Delete character |
| Any character | Type into input buffer |

### Workflow Examples

#### Adding a Task

1. Press `a` to enter add-task mode
2. Type your task description
3. Press `Enter` to add the task to the current folder

#### Organizing with Folders

1. Press `f` to enter folder mode
2. Type the folder name (e.g., "work", "personal", "shopping")
3. Press `Enter` to switch to that folder
4. Add tasks - they will be added to the current folder

#### Completing Tasks

1. Press `n` to enter navigation mode
2. Use `j`/`k` or arrow keys to select a task
3. Press `Space` to mark as done (or undone)
4. Press `Esc` to return to command mode

#### Deleting Tasks

1. Press `d` to enter delete mode
2. Type the task number
3. Press `Enter` to delete

## 📂 Data Storage

Tasks are automatically saved to:

```
~/.pastel_todo/tasks.txt
```

Each task is stored in the format:
```
[status]|folder|task_text|timestamp
```

Where:
- `status`: `[ ]` for incomplete, `[x]` for complete
- `folder`: The folder/category name
- `task_text`: Your task description
- `timestamp`: ISO 8601 creation timestamp

## 🎨 Color Scheme

The application uses a carefully selected pastel color palette:

- **Accent/Labels**: Pink (ANSI 219)
- **Task Text**: Light Blue (ANSI 153)
- **Completed Tasks**: Light Green (ANSI 151)
- **Dates**: Cream (ANSI 223)
- **Folders**: Mauve (ANSI 212)
- **Values**: Sky Blue (ANSI 159)
- **Borders**: Light Pink (ANSI 225)

## 🔧 Configuration

### Changing Terminal Size Requirements

Edit `src/main.rs` and modify these constants:

```rust
const MIN_WIDTH: u16 = 60;   // Minimum terminal width
const MIN_HEIGHT: u16 = 30;  // Minimum terminal height
```

Then rebuild:

```bash
cargo build --release
```

## 🛠️ Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Dependencies

- `crossterm` (0.27) - Cross-platform terminal manipulation
- `chrono` (0.4) - Date and time handling
- `dirs-next` (2) - Standard directory paths

## 🐛 Troubleshooting

### Terminal Size Error

**Error**: `Terminal must be at least 60x30 (current: XXxYY)`

**Solution**: Resize your terminal window to be at least 60 columns wide and 30 rows tall.

### Permission Denied

**Error**: Cannot write to `~/.pastel_todo/tasks.txt`

**Solution**: Ensure you have write permissions to your home directory:

```bash
mkdir -p ~/.pastel_todo
chmod 755 ~/.pastel_todo
```

### Colors Not Displaying

**Solution**: Ensure your terminal supports ANSI colors. Most modern terminals do. Try setting:

```bash
export TERM=xterm-256color
```

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Terminal UI powered by [crossterm](https://github.com/crossterm-rs/crossterm)
- Inspired by minimalist design principles

## 📞 Support

If you encounter any issues or have questions:

- Open an issue on GitHub
- Check existing issues for solutions
- Read the troubleshooting section above

---

**Made with 💜 and Rust**
