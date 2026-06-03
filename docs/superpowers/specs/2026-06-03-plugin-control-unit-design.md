# Plugin Control Unit Design

Status: frozen for implementation.

## Purpose

The Plugins tab treats each plugin package as the smallest controllable unit. Users enable or disable the plugin package, and bundled plugin Skills inherit that effective state in the GUI.

## Behavior

- The Plugins main table lists plugin packages only, one row per plugin.
- Selecting a plugin opens a secondary view for that plugin.
- The secondary view lists only `PluginProvidedSkill` capabilities from the selected plugin.
- Bundled Skills remain visible when their parent plugin is disabled.
- Bundled Skill status is derived from the parent plugin status:
  - parent `Enabled` -> Skill `Enabled`
  - parent `Disabled` -> Skill `Disabled`
  - parent `Invalid` -> Skill `Invalid`
  - parent `Unknown` -> Skill `Unknown`
- Bundled plugin Skills are read-only details. They cannot be individually enabled or disabled by renaming `SKILL.md`.

## Control Model

- `Enable Plugin` and `Disable Plugin` are the only state-changing controls for plugin-provided Skills.
- Plugin enablement writes Codex plugin configuration only:
  - `~/.codex/config.toml`
  - `[plugins."<plugin_key>"].enabled = true | false`
- Skill-kits does not mutate plugin package contents.
- Skill-kits does not rename plugin bundled `SKILL.md` files.

## UI Shape

Main Plugins view:

```text
Plugin          Provider         Agent   Status    Skills   Path   Updated
GitHub Tools    openai-curated   codex   Enabled   1 Skill  ...    ...
Browser         openai-bundled   codex   Disabled  2 Skills ...    ...
```

Plugin secondary view:

```text
Plugins / GitHub Tools

Skill          Status     Kind    Path
GitHub Skill   Enabled    Skill   ~/.codex/plugins/cache/...

Controls:
Back to Plugins
Disable Plugin
Scan Plugins
```

## Acceptance Criteria

- Plugin main rows no longer mix package rows and capability rows.
- Clicking a plugin shows its bundled Skills in a secondary view.
- Disabling a plugin makes the secondary Skill rows remain visible with `Disabled` status.
- Re-enabling a plugin makes the secondary Skill rows show `Enabled` status.
- Plugin enablement still updates only Codex plugin configuration.
- Plugin Skill package files are not modified.
