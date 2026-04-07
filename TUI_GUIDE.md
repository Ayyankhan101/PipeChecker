# 🎨 Pipecheck Interactive TUI Mode

## Overview

Pipecheck now includes an **interactive Terminal UI (TUI)** mode for a better visual experience when checking multiple workflows.

## Usage

```bash
pipecheck --tui
```

## Features

### 📋 Workflow List View
```
┌─────────────────────────────────────────────────────────────┐
│ 🔍 Pipecheck - Interactive Mode                            │
├─────────────────────────────────────────────────────────────┤
│ Workflows                                                   │
│ ▶ ✅ ci.yml          │ 0 errors │ 0 warnings               │
│   ✅ deploy.yml      │ 0 errors │ 0 warnings               │
│   ⚠️  security.yml   │ 0 errors │ 2 warnings               │
├─────────────────────────────────────────────────────────────┤
│ [↑/↓] Navigate  [Enter/Space] Details  [Q/Esc] Quit        │
└─────────────────────────────────────────────────────────────┘
```

### 📝 Details View
```
┌─────────────────────────────────────────────────────────────┐
│ 🔍 Pipecheck - Interactive Mode                            │
├─────────────────────────────────────────────────────────────┤
│ Details                                                     │
│ File: .github/workflows/ci.yml                             │
│ Provider: GitHubActions                                     │
│                                                             │
│ 0 errors, 0 warnings                                        │
│                                                             │
│ ✅ No issues found!                                         │
├─────────────────────────────────────────────────────────────┤
│ [↑/↓] Navigate  [Enter/Space] Back  [Q/Esc] Quit           │
└─────────────────────────────────────────────────────────────┘
```

### 🔴 Error Display
```
┌─────────────────────────────────────────────────────────────┐
│ Details                                                     │
│ File: .github/workflows/broken.yml                         │
│ Provider: GitHubActions                                     │
│                                                             │
│ 2 errors, 1 warnings                                        │
│                                                             │
│ ❌ ERROR: Circular dependency detected: job-a -> job-b     │
│    💡 Remove one of the dependencies to break the cycle    │
│                                                             │
│ ❌ ERROR: Job 'deploy' depends on non-existent job 'test'  │
│    💡 Remove dependency or add job 'test'                  │
│                                                             │
│ ⚠️  WARNING: Docker image uses :latest tag                 │
│    💡 Use specific version tags for reproducibility        │
└─────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` or `k` | Move up |
| `↓` or `j` | Move down |
| `Enter` or `Space` | Toggle details view |
| `q` or `Esc` | Quit |

## Status Indicators

- ✅ **Green checkmark** - No issues
- ⚠️  **Yellow warning** - Warnings found
- ❌ **Red X** - Errors found
- ❓ **Question mark** - Failed to audit

## Color Coding

- **Red** - Errors
- **Yellow** - Warnings
- **Blue** - Info messages
- **Green** - Success
- **Cyan** - Highlights and suggestions
- **Gray** - Help text

## When to Use TUI Mode

### Use TUI when:
- ✅ Checking multiple workflows
- ✅ You want visual feedback
- ✅ Exploring issues interactively
- ✅ Presenting to team members
- ✅ Learning about workflow issues

### Use CLI when:
- ✅ In CI/CD pipelines
- ✅ Scripting/automation
- ✅ Quick single-file checks
- ✅ JSON output needed
- ✅ Pre-commit hooks

## Examples

### Basic Usage
```bash
# Start TUI in current directory
cd your-project
pipecheck --tui
```

### With Options
```bash
# TUI with strict mode
pipecheck --tui --strict

# TUI without Docker checks
pipecheck --tui --no-docker
```

## Tips

1. **Navigation**: Use vim-style keys (`j`/`k`) or arrow keys
2. **Quick exit**: Press `q` or `Esc` anytime
3. **Details**: Press `Enter` or `Space` to see full error details
4. **Scrolling**: Details view automatically wraps long text

## Comparison: CLI vs TUI

| Feature | CLI | TUI |
|---------|-----|-----|
| Speed | ⚡ Instant | ⚡ Instant |
| Multiple files | List output | Interactive list |
| Error details | All at once | On-demand |
| Navigation | Scroll terminal | Arrow keys |
| CI/CD friendly | ✅ Yes | ❌ No |
| Visual appeal | Good | Excellent |
| Automation | ✅ Yes | ❌ No |
| Learning | Good | Better |

## Technical Details

- Built with [ratatui](https://github.com/ratatui-org/ratatui) - Modern Rust TUI framework
- Uses [crossterm](https://github.com/crossterm-rs/crossterm) for terminal control
- Supports all major terminals (iTerm2, Terminal.app, Windows Terminal, etc.)
- Works over SSH
- Respects terminal colors and themes

## Troubleshooting

### TUI doesn't start
```bash
# Check if terminal supports TUI
echo $TERM

# Should output something like: xterm-256color
```

### Colors look wrong
Your terminal might not support 256 colors. Try:
```bash
export TERM=xterm-256color
pipecheck --tui
```

### Keyboard shortcuts don't work
Make sure your terminal is in focus and not capturing the keys.

## Future Enhancements

Coming in future versions:
- 🔄 Real-time updates (watch mode in TUI)
- 📊 Workflow visualization
- 🔍 Search/filter workflows
- 📈 Statistics dashboard
- 🎨 Custom themes
- ⌨️  More keyboard shortcuts
- 📋 Copy error messages

## Feedback

Love the TUI? Have suggestions? Open an issue on GitHub!

---

**Pipecheck TUI - Making workflow validation beautiful! 🎨**
