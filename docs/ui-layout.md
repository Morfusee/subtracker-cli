# UI Layout Phases — Formal Terms

Source of truth: `src/ui/mod.rs:23` (`LayoutMode`), `Density`, and `src/ui/provider_card.rs`.

All width thresholds are in terminal columns. Mode is selected by `LayoutMode::for_width(width)`.

| Phase | Canonical name | Shorthand | Width range | Alias for conversation | Intent |
|-------|---------------|-----------|-------------|------------------------|--------|
| P3 | `Wide` | `W` | `>= 100` | "desktop / wide" | Full fidelity. No clipping. |
| P2 | `Compact` | `C` | `70 – 99` | "tablet / compact" — **first breakpoint below `W`** | First snap. Content stays visible. Group is `mx-auto` centered as a block (not per-line `text-align:center`). |
| P1 | `Narrow` | `N` | `< 70` | "mobile / narrow" | Most compressed. Multiline fallback. |

> Say `W`, `C`, or `N` in reviews. Example: "Center `C` via `mx-auto`, leave `W` left-indented, keep `N` stacked."

## Thresholds and responsive density

```rust
// src/ui/mod.rs:35
pub const fn for_width(width: u16) -> Self {
    if width >= 100 { Self::Wide }        // P3/W
    else if width >= 70 { Self::Compact } // P2/C — first media query
    else { Self::Narrow }                 // P1/N
}
```

`MIN_WIDTH = 60`, `MIN_HEIGHT = 20` remain defined but **no longer block rendering** — `render()` clamps `total_required_height = total_required_height.min(area.height)` and always attempts to draw.

## Height-aware density

Width selects `Wide`, `Compact`, or `Narrow`. The renderer then measures four density candidates using generated card lines:

- `Normal`: horizontal card padding `3`, 1-row gaps, interior blank lines, generous bars, STC header shown (requires `>= 35` rows).
- `Compact`: horizontal card padding `2`, 1-row gaps, zero interior blank lines, inline single-line quotas, medium bars, STC header shown (requires `>= 24` rows).
- `Spaced`: for narrow heights (`< 24` rows). Hides the STC header to save 7 rows, while **strictly enforcing spacing**: card padding `2`, 1-row card gaps, 1-row footer gap, clean single-column quotas (fits `>= 18` rows).
- `Dense`: for ultra-narrow heights (`< 18` rows). Horizontal card padding `1`, zero card gaps, shortest footer (fits `>= 15` rows).

The first candidate whose content width and complete required height fit the terminal is rendered. If none fit completely, `Dense` is rendered into the available area.

## Per-mode rendering (current)

### Quota cards (`Codex`, `Antigravity` → `src/ui/provider_card.rs` `quota_lines`)

- Inline single-line format (`label`, `bar`, and `◷ reset`) in `Wide` and `Compact` modes.
- **P3/W** (`Wide`): `bar_w = inner_width - 48` clamped `16..60` (`12..40` in Compact/Spaced, `6..24` in Dense), `◷ reset` inline.
- **P2/C** (`Compact`): `bar_w = inner_width - 44` clamped `12..28` (`8..20` in Compact/Spaced, `4..14` in Dense), `◷ reset` inline.
- **P1/N** (`Narrow`): Dedicated 2-line layout per quota (`Line 1: label`, `Line 2: │bar│ % reset in {time}`) so bars never sink into labels; `bar_w = inner_width - 26` clamped `8..24` (`6..20` in Compact/Spaced, `4..12` in Dense).
- In `Normal`, `Compact`, and `Spaced` densities, blank lines are placed between each label-bar element to ensure generous breathing room.

### OpenCode stats (`src/ui/provider_card.rs` `open_code_lines`)

- **P3/W**: 2-column grid via `stat_grid_line` — `Sessions | Input` / `Total Cost | Output`.
- **P2/C**: 2-column grid in `Compact`/`Spaced`/`Dense` when `inner_width >= 56`, otherwise 4 stacked centered rows.
- **P1/N**: 4 stacked rows with `4` (or `2` in Dense) spaces indent.
- In `Normal` density, 1 blank line top/bottom; in `Compact`, `Spaced`, and `Dense`, decorative blank lines are omitted.

### Footer / header

- Header (STC banner): Shown in `Normal` and `Compact` densities (heights `>= 24`); hidden in `Spaced` and `Dense` (narrow heights `< 24`) to enforce card spacing.
- Footer bindings and responsive strings are defined under `Keyboard interaction and collapsed cards` below.

### Keyboard interaction and collapsed cards

- Codex is focused at startup.
- `j`/`Down` and `k`/`Up` move focus with wraparound.
- `Space`/`Enter` toggle the focused card.
- Focus is indicated by inverted title styling and a bold card border.
- A collapsed card is two rows high: provider title on the top border and status on the bottom border.
- Collapse hides body data by explicit user choice; responsive density alone never omits data.

Footer strings:

- `W`: `[j/k/↑/↓] select   [Space/Enter] collapse   [r] refresh   [q] quit`
- `C`: `[j/k/↑/↓] select  [Space/Enter] toggle  [r] refresh  [q] quit`
- `N`: `[j/k] move  [Space] toggle  [r] refresh  [q] quit`

## Invariants for future agents

1. **Logo shown when height >= 24, hidden on narrow heights to preserve spacing** — on narrow heights (< 24 rows), hide the logo so card gaps and padding remain intact without squishing or clipping data.
2. **Never collapse label-bar elements into a single line on narrow mode** — keep the label on its own row so bars never sink into labels.
3. **Always enforce vertical breathing room between quota elements** — maintain blank spacing between label-bar items.
4. **Never block rendering on small terminals** — `render()` must not early-return with a "too small" message; it clamps and draws.
5. **Responsive density removes spacing and condenses layouts, not data** — provider names, labels, percentages, reset times, and statistics are never omitted.
6. Keep `P3/W`, `P2/C`, `P1/N` names stable. If you rename the enum, update this doc, `README.md`, and all call sites.

## Quick check

```text
cargo test --lib ui::tests::layout_breakpoints_match_the_design
cargo test --lib ui::tests::logo_is_present_on_standard_height_and_hidden_on_narrow_height
cargo test --lib ui::provider_card::tests::quota_elements_have_spacing_and_border_caps
cargo test --lib ui::provider_card::tests::wide_quota_row_has_bar_percentage_remaining_and_reset  # must assert !contains("remaining")
cargo test --lib ui::tests::height_constrained_dashboard_keeps_all_provider_data
cargo test --lib ui::tests::absolute_minimum_size_has_deliberate_fallback_copy  # must assert dashboard renders at 50×15
```
