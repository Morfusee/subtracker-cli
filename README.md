# Subtracker

Subtracker is a small terminal dashboard for current AI coding-tool usage.

Run it as:

```text
stc
```

## Providers

### Codex

Subtracker reads the currently authenticated Codex session from:

```text
~/.codex/auth.json
```

and queries the first-party ChatGPT Codex usage backend.

Subtracker does not refresh OAuth credentials. If the Codex session has expired:

```text
codex login
```

The `wham/usage` backend is a first-party but undocumented integration point and may change. The Codex adapter is isolated so it can be replaced by the Codex app-server interface later.

### OpenCode

Subtracker reads your OpenCode API credentials from:

```text
~/.local/share/opencode/auth.json
```

and queries the OpenCode Go subscription usage endpoint:

```text
GET https://opencode.ai/zen/go/v1/usage
```

Subtracker monitors:

- 5 hour rolling limit
- Weekly limit
- Monthly limit

### Antigravity

Antigravity CLI must already be installed and authenticated.

Subtracker invokes:

```text
agy -p "/usage" --output-format json
```

It does not read Antigravity OAuth credentials directly.

## Controls

```text
r       refresh immediately
q       quit
Ctrl+C  quit
```

Providers refresh automatically every 60 seconds.

## Installation

### Via Cargo (crates.io)

```bash
cargo install subtracker
```

This installs the binary `stc` to your Cargo bin folder (`~/.cargo/bin`), allowing you to run `stc` from any terminal.

### Direct Download from Releases

Download the precompiled standalone executable (`stc.exe` on Windows or `stc-*.tar.gz` on macOS) matching your platform from [GitHub Releases](https://github.com/Morfusee/subtracker-cli/releases), and place it in a directory on your `PATH`.

## Build from source

Install the Rust stable toolchain, then:

```text
cargo build --release
```

The executable is:

```text
target/release/stc
```

or on Windows:

```text
target\release\stc.exe
```

## Appearance

The TUI uses provider accent colors and semantic quota colors. It keeps the terminal's existing background and requires no Nerd Font.

Subtracker also respects the conventional `NO_COLOR` environment variable:

macOS:

```text
NO_COLOR=1 stc
```

Windows PowerShell:

```text
$env:NO_COLOR = "1"
stc
```

Responsive modes — formal phases (see `docs/ui-layout.md`):

```text
P3/W  Wide      >= 100 columns  — desktop, full fidelity (flex-row)
P2/C  Compact   70–99 columns   — first breakpoint, Grid auto-fit/expand (flex-col per quota, side-by-side when space allows)
P1/N  Narrow    < 70 columns    — mobile, single-column flex-col
```

Small terminals are never blocked: `render()` clamps `total_required_height` to `area.height` and draws truncated content instead of a "terminal too small" message. `Wide` no longer shows the `remaining` label after its bar (removed 2026-08-29).

## v1 boundaries

Subtracker does not store provider credentials, manage accounts, refresh Codex OAuth, persist usage history, run a daemon, or provide a desktop/tray UI.

