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

## Install from a release

Before placing `stc` on your PATH, check whether another installed tool already uses that command name.

macOS:

```text
command -v stc
```

Windows PowerShell:

```text
Get-Command stc -ErrorAction SilentlyContinue
```

Download the archive matching your platform from GitHub Releases, extract `stc`/`stc.exe`, and place it in a directory on your PATH.

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

Responsive modes:

```text
Wide      >= 100 columns
Compact   70-99 columns
Narrow    < 70 columns
```

Below 60 columns or 20 rows, Subtracker renders a terminal-too-small message. If the current provider data needs more vertical rows than are available, it asks for additional height rather than clipping the dashboard.

## v1 boundaries

Subtracker does not store provider credentials, manage accounts, refresh Codex OAuth, persist usage history, run a daemon, or provide a desktop/tray UI.

