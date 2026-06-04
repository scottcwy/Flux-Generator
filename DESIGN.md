# Skill-kits Design Context

## Design Direction

Skill-kits uses a Monochrome Local Workbench style: near-black, precise, compact, and grid-driven. The design should borrow the restraint of Codex and Vercel-like product interfaces without copying either brand.

## Scene

A developer opens the app on macOS during focused project work. They are scanning dense operational state, comparing rows, checking paths, and deciding whether an enable, disable, scan, adopt, or project action is safe. The environment favors a dark theme with high contrast, quiet surfaces, and fast hover feedback.

## Color Strategy

Restrained monochrome. Black, gray, and near-white carry the product identity. Semantic colors are rare and operational.

Core dark tokens:

| Token | Hex | Use |
| --- | --- | --- |
| `canvas` | `#050505` | App background |
| `surface_1` | `#050505` | Sidebar and base panels |
| `surface_2` | `#161616` | Hover rows and secondary controls |
| `surface_3` | `#1f1f1f` | Selected rows and active controls |
| `surface_4` | `#2a2a2a` | Strong focus surfaces |
| `hairline` | `#242424` | Default dividers |
| `hairline_strong` | `#3a3a3a` | Active outlines |
| `ink` | `#ededed` | Primary text |
| `ink_muted` | `#a1a1a1` | Secondary text |
| `ink_subtle` | `#737373` | Metadata and helper text |
| `ink_tertiary` | `#525252` | Disabled text |

Semantic tokens:

| Token | Hex | Use |
| --- | --- | --- |
| `success` | `#67a878` | Enabled or healthy |
| `warning` | `#c5a365` | Caution, drift, outdated |
| `danger` | `#d06b6b` | Invalid or destructive |
| `info` | `#a3a3a3` | Neutral informational states |
| `focus` | `#ededed` | Keyboard focus and selected outline |

## Typography

Use platform-native UI fonts. Use monospace only for paths, hashes, IDs, commands, and registry snippets. Keep letter spacing at zero.

Type scale:

| Role | Size | Weight |
| --- | --- | --- |
| Page title | 20 | 600 |
| Section heading | 15 | 600 |
| Body | 13 | 400 |
| Strong body | 13 | 500 |
| Caption | 12 | 400 |
| Mono | 12 | 400 |
| Button | 13 | 500 |

## Layout

Use a fixed desktop shell:

- Sidebar: primary navigation and recent project scope switcher.
- Main pane: workbench table or dashboard rows.
- Inspector: selected object details and controls.

The design should favor strict x-axis alignment, stable row heights, and predictable spacing. Avoid nested cards and floating section cards.

## Component Rules

- Sidebar items should behave like tab rows: fixed height, aligned icon column, aligned label, clear active fill.
- Tables use whole-row hover and selection.
- Hover uses `surface_2`; selected uses `surface_3`.
- Read-only starts quieter than hover and must not look stuck in hover state.
- Badges include icon or text, not color alone.
- Dividers align with section content and should not create uneven left/right fragments.
- Path controls should use icon plus text or icon buttons with tooltips where space is tight.

## Bans

Do not use lavender brand accents, purple gradients, glows, glassmorphism, large rounded cards, pill-shaped primary buttons, repeated icon cards, marketing copy, or hero metric layouts.
