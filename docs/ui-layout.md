# UI Layout Phases — Formal Terms

Source of truth: `src/ui/mod.rs:21` (`LayoutMode`) and `src/ui/provider_card.rs`.

All width thresholds are in terminal columns. Mode is selected by `LayoutMode::for_width(width)`.

| Phase | Canonical name | Shorthand | Width range | Alias for conversation | Intent |
|-------|---------------|-----------|-------------|------------------------|--------|
| P3 | `Wide` | `W` | `>= 100` | "desktop / wide" | Full fidelity. No clipping. |
| P2 | `Compact` | `C` | `70 – 99` | "tablet / compact" — **first breakpoint below `W`** | First snap. Content stays visible. Group is `mx-auto` centered as a block (not per-line `text-align:center`). |
| P1 | `Narrow` | `N` | `< 70` | "mobile / narrow" | Most compressed. Multiline fallback. |

> Say `W`, `C`, or `N` in reviews. Example: "Center `C` via `mx-auto`, leave `W` left-indented, keep `N` stacked."

## Thresholds (code)

```rust
// src/ui/mod.rs:28
pub const fn for_width(width: u16) -> Self {
    if width >= 100 { Self::Wide }      // P3/W
    else if width >= 70 { Self::Compact } // P2/C — first media query
    else { Self::Narrow }                // P1/N
}
```

`content_width = area.width.min(120)` then `inner_width = content_width - 8` (borders + `Padding::horizontal(3)`). `Compact` centering uses `inner_width` to compute `pad = (inner_width - max_row_width)/2`.

`MIN_WIDTH = 60`, `MIN_HEIGHT = 20` remain defined but **no longer block rendering** — since 2026-08-29 `render()` clamps `total_required_height = total_required_height.min(area.height)` and always attempts to draw (see `src/ui/mod.rs:72`). Helpers `render_minimum_size_message` / `render_content_too_tall_message` were removed.

## Per-mode rendering (current)

### Quota cards (`Codex`, `Antigravity` → `src/ui/provider_card.rs:103` `quota_lines`)

- **P3/W** (`Wide`): `label 20` / `bar` (expanded `bar_w = inner-33` clamp `20..80`, takes remaining service-box width) / `◷ reset` under bar, `4` left indent + `1` right pad per line, **1 blank line between quota items**. **No `remaining` text** (removed 2026-08-29). Longest label (`Claude/GPT weekly` 17) still fits in 20.
- **P2/C** (`Compact`): **Grid** (`auto-fit`, `auto-expand`) — each quota is a `flex-col` item (`label` / `bar` / `◷ reset`). Items are placed in a grid with `gap 6`, `cell_pad 1` on each side, `min_item 26`, `cols = (inner_width+gap)/(min_item+gap)` clamped `1..3` (`col_width = (inner_width - gap*(cols-1))/cols`, `content_width = col_width-2`, `bar_w = content_width-7` clamp `8..20`). Cells have `1` char padding on each side and `1` blank line between grid rows. Grid expands to fill `inner_width`.
- **P1/N** (`Narrow`): `flex-col` per quota — `label` / `bar` (`inner_width-18` clamp `8..24`) / `resets in …` — `4` left indent + `1` right pad per line, left-aligned, single column, **1 blank line between quota items** (lessened from `5` left).

### OpenCode stats (`src/ui/provider_card.rs:206` `open_code_lines`)

- **P3/W**: 2-column grid via `stat_grid_line` — `Sessions | Input` / `Total Cost | Output`.
- **P2/C**: 4 stacked rows (`label 14` + value), **same `mx-auto` block** logic as quota `C` — shared pad, left-aligned inside centered block.
- **P1/N**: 4 stacked rows with `4` spaces indent, left-aligned (`stat_line` `label 12`).

### Footer / header

- Header (STC banner) only if `area.height >= cards_required_height + 7`; otherwise hidden. No blocking.
- Footer strings per mode (`src/ui/mod.rs:183`):
  - `W`: `[r] refresh        ◷ auto 60s        [q] quit        [Ctrl+C] exit`
  - `C`: `[r] refresh   60s auto   [q] quit   [Ctrl+C] exit`
  - `N`: `[r] refresh   [q] quit   [^C] exit`

## How to talk about changes

- "Tweak `C` grid" → edit `cols`/`col_width`/`bar_w`/`gap`/`cell_pad` in `provider_card.rs:168` and grid generation `provider_card.rs:180`; vertical gap is the extra `Line::from("")` between chunks.
- "Fix `C` label touching bar" → `C` is now grid flex-col per item, so touching is impossible by construction.
- "Change `W` quota density" → edit `LayoutMode::Wide` arm in `quota_lines`.
- "Adjust breakpoints" → change `for_width` thresholds and update this doc + `README.md` + tests `layout_breakpoints_match_the_design`.

## Invariants for future agents

1. **Never re-add `remaining` to `W` without explicit request** — removed per 2026-08-29.
2. **Never block rendering on small terminals** — `render()` must not early-return with a "too small" message; it clamps and draws.
3. **`C` is the first media query** — `W → C` is the snap the user refers to. `C` must use **Grid** (auto-fit/expand), not single-column `mx-auto` or per-line `Alignment::Center`.
4. **`C` grid** — `label`/`bar`/`◷ reset` flex-col per item, `gap 6`, `cell_pad 1` each side, `min_item 26`, `cols = (inner+gap)/(min+gap)` (`1..3`), `content_width = col_width-2`, `bar_w = content_width-7`. Vertical `1` blank line between grid rows. `N` stays single-column. Do not revert `C` to flex-col single column without grid/padding.
5. Keep `P3/W`, `P2/C`, `P1/N` names stable. If you rename the enum, update this doc, `README.md`, and all call sites.

## Quick check

```text
cargo test --lib ui::tests::layout_breakpoints_match_the_design
cargo test --lib ui::provider_card::tests::wide_quota_row_has_bar_percentage_remaining_and_reset  # must assert !contains("remaining")
cargo test --lib ui::tests::absolute_minimum_size_has_deliberate_fallback_copy  # must assert dashboard renders at 50×15
```
