# Subtracker (`stc`)

<p align="center">
  <a href="https://crates.io/crates/subtracker"><img src="https://img.shields.io/crates/v/subtracker.svg?style=flat-square&color=blue" alt="Crates.io Version"></a>
  <a href="https://github.com/Morfusee/subtracker-cli/releases"><img src="https://img.shields.io/github/v/release/Morfusee/subtracker-cli?style=flat-square&color=green" alt="GitHub Release"></a>
  <a href="https://github.com/Morfusee/subtracker-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/Morfusee/subtracker-cli/ci.yml?branch=main&style=flat-square" alt="CI Status"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://crates.io/crates/subtracker"><img src="https://img.shields.io/crates/d/subtracker.svg?style=flat-square&color=orange" alt="Crates.io Downloads"></a>
</p>

<p align="center">
  <strong>A fast, zero-dependency, responsive terminal dashboard for tracking AI subscription quotas and usage in real-time.</strong>
</p>

```text
  ███████╗████████╗ ██████╗ 
  ██╔════╝╚══██╔══╝██╔════╝ 
  ███████╗   ██║   ██║      
  ╚════██║   ██║   ██║      
  ███████║   ██║   ╚██████╗ 
  ╚══════╝   ╚═╝    ╚═════╝ 

╭──  CODEX  ───────────────────────────────────────────────────────────────────╮
│                                                                              │
│      5 hour             │▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░│   65%   ◷ 1h 0m               │
│                                                                              │
│      Weekly             │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░│   79%   ◷ 6d 0h               │
│                                                                              │
╰──────────────────────────────────────────────────────  ● updated just now  ──╯
╭──  OPENCODE  ────────────────────────────────────────────────────────────────╮
│                                                                              │
│      Sessions    2,277             │  Input     312.3M tokens                │
│      Total Cost  $120.50           │  Output    15.3M tokens                 │
│                                                                              │
╰──────────────────────────────────────────────────────  ● updated just now  ──╯
╭──  ANTIGRAVITY  ─────────────────────────────────────────────────────────────╮
│                                                                              │
│      Gemini 5 hour      │▓▓▓▓▓▓▓▓▓░░░░░░░░░░░│   45%   ◷ 3m                  │
│                                                                              │
│      Gemini weekly      │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░│   90%   ◷ 6d 0h               │
│                                                                              │
│      Claude/GPT 5 hour  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  100%   ◷ 5h 0m               │
│                                                                              │
│      Claude/GPT weekly  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  100%   ◷ 7d 0h               │
│                                                                              │
╰──────────────────────────────────────────────────────  ● updated just now  ──╯
 [r] refresh               ◷ auto 60s               [q] quit            [Ctrl+C] exit
```

---

## ✨ Features

- ⚡ **Zero External Dependencies** — Single standalone native binary compiled in Rust with Ratatui.
- 🎯 **Multi-Provider Support** — Monitors quotas and rate limits across:
  - **Codex (ChatGPT / GitHub Copilot)**: 5-hour rolling limit, weekly limit, and reset countdowns.
  - **OpenCode**: Session count, cumulative cost ($), input tokens, and output tokens.
  - **Google Antigravity**: Gemini and Claude/GPT 5-hour and weekly quotas.
- 📐 **Adaptive Responsive Scaling** — Seamlessly adjusts layout density across Wide (desktop), Compact, and Narrow (split-pane/mobile) terminal windows with zero clipping.
- 🔄 **Live Background Polling** — Automatically refreshes usage every 60 seconds with live spinners and relative timestamps (`● updated just now`).
- 🎨 **TrueColor & NO_COLOR Support** — Clean visual aesthetics with semantic health colors and provider accents; respects the `NO_COLOR` standard.

---

## 📦 Installation

### Option 1: Via Cargo (crates.io) — Recommended

If you have Rust installed, install `subtracker` directly from [crates.io](https://crates.io/crates/subtracker):

```bash
cargo install subtracker
```

*This compiles and places the `stc` binary into your Cargo bin directory (`~/.cargo/bin`).*

---

### Option 2: Pre-compiled Binaries (macOS & Windows)

Download the standalone binary directly from the latest [GitHub Releases](https://github.com/Morfusee/subtracker-cli/releases/latest):

#### **macOS (Apple Silicon M1/M2/M3/M4):**
```bash
curl -sSL https://github.com/Morfusee/subtracker-cli/releases/latest/download/stc-aarch64-apple-darwin.tar.gz | tar -xz
sudo mv stc /usr/local/bin/
```

#### **macOS (Intel):**
```bash
curl -sSL https://github.com/Morfusee/subtracker-cli/releases/latest/download/stc-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv stc /usr/local/bin/
```

#### **Windows (PowerShell):**
Download [`stc.exe`](https://github.com/Morfusee/subtracker-cli/releases/latest/download/stc.exe) and move it to a folder in your `PATH`:
```powershell
Invoke-WebRequest -Uri "https://github.com/Morfusee/subtracker-cli/releases/latest/download/stc.exe" -OutFile "$HOME\.cargo\bin\stc.exe"
```

---

### Option 3: Build from Source

```bash
git clone https://github.com/Morfusee/subtracker-cli.git
cd subtracker-cli
cargo build --release
```

The compiled binary will be located at:
- **macOS / Linux:** `target/release/stc`
- **Windows:** `target\release\stc.exe`

---

## 🚀 Usage & Keyboard Controls

Launch Subtracker by running:

```bash
stc
```

| Key | Action |
| :--- | :--- |
| **`r`** | Trigger an immediate manual refresh across all providers |
| **`q`** | Quit Subtracker and cleanly restore the terminal |
| **`Ctrl+C`** | Exit immediately |

---

## 🔌 Supported Providers

### 1. Codex
Subtracker reads your authenticated Codex session from `~/.codex/auth.json` and queries the ChatGPT Codex usage endpoint.

If your session has expired, simply log in again via:
```bash
codex login
```

### 2. OpenCode
Subtracker reads your API credentials from `~/.local/share/opencode/auth.json` (or OS equivalent) and queries the OpenCode Go subscription usage API:
- Sessions count
- Total monthly cost
- Input & Output token counters

### 3. Antigravity
Subtracker connects to the local Antigravity CLI via:
```bash
agy -p "/usage" --output-format json
```

---

## 📐 Layout & Responsive Modes

Subtracker automatically calculates the available terminal geometry and dynamically adjusts layout density:

| Mode | Width | Behavior |
| :--- | :--- | :--- |
| **`Wide` (`P3/W`)** | `≥ 100` cols | Full desktop fidelity; side-by-side stats grid and expanded progress bars. |
| **`Compact` (`P2/C`)** | `70..99` cols | Balanced layout; inline quota metrics and stacked stats. |
| **`Narrow` (`P1/N`)** | `< 70` cols | Dedicated 2-line layout per quota (`Line 1: label`, `Line 2: bar + reset`) preventing label squishing. |

On height-constrained windows (`< 24` rows), the ASCII header is automatically hidden to ensure all provider cards and spacing remain intact with zero truncation.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
