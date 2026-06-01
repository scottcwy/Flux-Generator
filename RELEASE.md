# Skill-kits v0.1 macOS single binary checklist

This checklist is for a local macOS release build of the `skill-kits` binary.

## Verify

Run from the repository root:

```bash
rtk /opt/homebrew/bin/cargo fmt --all --check
rtk /opt/homebrew/bin/cargo clippy --all-targets --all-features -- -D warnings
rtk /opt/homebrew/bin/cargo test
```

## Build

```bash
rtk /opt/homebrew/bin/cargo build --release
```

The release binary is:

```text
target/release/skill-kits
```

## Runtime

The v0.1 macOS release is a single Rust binary. It must not require a Node.js or
Python runtime to list, install, deploy, adopt, scan, run doctor checks, or open
the native GUI.

## GUI Smoke

Run GUI smoke checks with an isolated home so local release testing does not
mutate a real `~/.skill-kits` directory:

```bash
rtk mkdir -p /tmp/skill-kits-smoke-home
rtk env HOME=/tmp/skill-kits-smoke-home target/release/skill-kits --gui
```

Manual acceptance:

- Dashboard opens first in Global Inventory scope with Dashboard, Skills,
  Agents, and Projects navigation in that order.
- The app uses the dark monochrome workbench style, compact tables, a sidebar,
  and a right inspector without marketing panels or gradient/card-heavy UI.
- Empty Skills and Projects states explain the next safe action.
- Clicking Refresh, Adopt all, Import as new, Skip, Scan, Deploy, Enable,
  Disable, Redeploy, Overwrite, Promote, Remove, and Uninstall either completes
  the action or shows a visible success/error message.
- Destructive remove with local drift requires confirmation and states that only
  the deployed Skill directory is removed.
- Projects shows outdated, drift, invalid toggle, and missing managed source
  states with only safe actions available for missing managed source.
- Agents view does not imply global Agent sync or unsupported v0.1 runtime
  behavior.
