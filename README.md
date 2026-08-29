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

OpenCode must already be installed and usable:

```text
opencode stats
```

Subtracker shows the v1 subset:

- sessions
- total cost
- input tokens
- output tokens

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

## v1 boundaries

Subtracker does not store provider credentials, manage accounts, refresh Codex OAuth, persist usage history, run a daemon, or provide a desktop/tray UI.
