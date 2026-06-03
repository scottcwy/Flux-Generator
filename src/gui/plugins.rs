use crate::core::plugins::{
    PluginPackage, PluginRuntimeCapability, PluginStatus, RuntimeCapabilityKind,
};
use crate::gui::state::{GuiModel, InspectorSection, RenderRow, RenderableView};

pub fn view_name() -> &'static str {
    "Plugins"
}

pub fn renderable(model: &GuiModel) -> RenderableView {
    if let Some(plugin) = model.selected_plugin() {
        return plugin_skill_renderable(model, plugin);
    }

    let mut main_rows = Vec::new();
    for plugin in &model.plugins {
        main_rows.push(RenderRow {
            id: plugin.id.clone(),
            cells: plugin_cells(plugin),
        });
    }

    RenderableView {
        view: model.active_view,
        title: view_name().to_string(),
        columns: vec![
            "Plugin".to_string(),
            "Provider".to_string(),
            "Agent".to_string(),
            "Status".to_string(),
            "Skills".to_string(),
            "Path".to_string(),
            "Updated".to_string(),
        ],
        main_rows,
        inspector_sections: inspector_sections(model),
        empty_message: model
            .plugins
            .is_empty()
            .then_some("No Codex plugins found. Scan ~/.codex/plugins/cache for plugin packages."),
    }
}

fn plugin_skill_renderable(model: &GuiModel, plugin: &PluginPackage) -> RenderableView {
    let main_rows = plugin
        .capabilities
        .iter()
        .filter(|capability| capability.kind == RuntimeCapabilityKind::PluginProvidedSkill)
        .map(|capability| RenderRow {
            id: capability.id.clone(),
            cells: plugin_skill_cells(plugin, capability),
        })
        .collect::<Vec<_>>();

    RenderableView {
        view: model.active_view,
        title: format!("Plugins / {}", plugin.display_name),
        columns: vec![
            "Skill".to_string(),
            "Status".to_string(),
            "Kind".to_string(),
            "Path".to_string(),
        ],
        main_rows,
        inspector_sections: inspector_sections(model),
        empty_message: Some("No plugin-provided Skills found."),
    }
}

fn plugin_cells(plugin: &PluginPackage) -> Vec<String> {
    vec![
        plugin.display_name.clone(),
        plugin.provider.clone(),
        plugin.agent_id.to_string(),
        status_label(&plugin.status).to_string(),
        plugin_skill_summary(plugin),
        plugin.package_path.to_string(),
        plugin.updated_at.clone().unwrap_or_else(|| "-".to_string()),
    ]
}

fn plugin_skill_cells(plugin: &PluginPackage, capability: &PluginRuntimeCapability) -> Vec<String> {
    vec![
        capability.name.clone(),
        status_label(&plugin.status).to_string(),
        runtime_kind_label(&capability.kind).to_string(),
        capability.path.to_string(),
    ]
}

fn inspector_sections(model: &GuiModel) -> Vec<InspectorSection> {
    if let Some(capability) = model.selected_plugin_capability() {
        return capability_sections(model, capability);
    }
    let Some(plugin) = model.selected_plugin().or_else(|| model.plugins.first()) else {
        return vec![InspectorSection {
            title: "Empty".to_string(),
            lines: vec![
                "No Codex plugins found.".to_string(),
                "Scan ~/.codex/plugins/cache for plugin packages.".to_string(),
            ],
        }];
    };
    plugin_sections(model, plugin)
}

fn plugin_sections(model: &GuiModel, plugin: &PluginPackage) -> Vec<InspectorSection> {
    vec![
        InspectorSection {
            title: "Summary".to_string(),
            lines: vec![
                plugin.display_name.clone(),
                format!("Plugin id {}", plugin.id),
                format!("Plugin key {}", plugin.plugin_key),
                format!("Agent {}", plugin.agent_id),
                format!("Provider {}", plugin.provider),
                format!("Status {}", status_label(&plugin.status)),
                format!("Can toggle {}", if plugin.can_toggle { "Yes" } else { "No" }),
            ],
        },
        InspectorSection {
            title: "Paths".to_string(),
            lines: vec![
                format!("Package path {}", plugin.package_path),
                plugin
                    .manifest_path
                    .as_ref()
                    .map(|path| format!("Manifest path {path}"))
                    .unwrap_or_else(|| "Manifest path -".to_string()),
            ],
        },
        InspectorSection {
            title: "Capabilities".to_string(),
            lines: capability_lines(plugin),
        },
        InspectorSection {
            title: "Config".to_string(),
            lines: vec![
                format!("Config path {}", model.codex_plugin_config_path()),
                "This plugin is managed through Codex plugin configuration. Skill-kits does not modify plugin package contents.".to_string(),
            ],
        },
        InspectorSection {
            title: "Actions".to_string(),
            lines: vec![
                "Scan Plugins refreshes Codex plugin package discovery.".to_string(),
                "Enable Plugin and Disable Plugin write Codex plugin configuration only.".to_string(),
            ],
        },
    ]
}

fn capability_sections(
    model: &GuiModel,
    capability: &PluginRuntimeCapability,
) -> Vec<InspectorSection> {
    let parent = model
        .plugins
        .iter()
        .find(|plugin| plugin.id == capability.parent_plugin_id);
    let mut summary = vec![
        capability.name.clone(),
        format!("Capability id {}", capability.id),
        format!("Kind {}", runtime_kind_label(&capability.kind)),
        format!(
            "Read-only {}",
            if capability.read_only { "Yes" } else { "No" }
        ),
    ];
    if let Some(parent) = parent {
        summary.push(format!("Plugin key {}", parent.plugin_key));
    }

    vec![
        InspectorSection {
            title: "Summary".to_string(),
            lines: summary,
        },
        InspectorSection {
            title: "Paths".to_string(),
            lines: vec![format!("Path {}", capability.path)],
        },
        InspectorSection {
            title: "State".to_string(),
            lines: capability_state_lines(capability),
        },
        InspectorSection {
            title: "Actions".to_string(),
            lines: vec!["Runtime capability rows are read-only.".to_string()],
        },
    ]
}

fn capability_state_lines(capability: &PluginRuntimeCapability) -> Vec<String> {
    if capability.kind == RuntimeCapabilityKind::PluginProvidedSkill {
        vec!["This Skill is bundled by a Codex plugin. It is not a native Agent Space Skill and cannot be enabled or disabled by renaming SKILL.md.".to_string()]
    } else {
        vec!["This runtime capability is managed by its parent plugin.".to_string()]
    }
}

fn capability_lines(plugin: &PluginPackage) -> Vec<String> {
    if plugin.capabilities.is_empty() {
        return vec!["No capabilities".to_string()];
    }
    plugin
        .capabilities
        .iter()
        .map(|capability| {
            format!(
                "{} {}",
                runtime_kind_label(&capability.kind),
                capability.name
            )
        })
        .collect()
}

fn status_label(status: &PluginStatus) -> &'static str {
    match status {
        PluginStatus::Enabled => "Enabled",
        PluginStatus::Disabled => "Disabled",
        PluginStatus::Unknown => "Unknown",
        PluginStatus::Invalid => "Invalid",
    }
}

fn runtime_kind_label(kind: &RuntimeCapabilityKind) -> &'static str {
    match kind {
        RuntimeCapabilityKind::PluginProvidedSkill => "Skill",
        RuntimeCapabilityKind::Command => "Command",
        RuntimeCapabilityKind::Agent => "Agent",
        RuntimeCapabilityKind::Asset => "Asset",
        RuntimeCapabilityKind::App => "App",
        RuntimeCapabilityKind::Unknown => "Unknown",
    }
}

fn capabilities_summary(capabilities: &[PluginRuntimeCapability]) -> String {
    if capabilities.is_empty() {
        return "No capabilities".to_string();
    }
    let mut counts = std::collections::BTreeMap::new();
    for capability in capabilities {
        *counts
            .entry(runtime_kind_label(&capability.kind))
            .or_insert(0usize) += 1;
    }
    ["Skill", "Command", "Agent", "Asset", "App", "Unknown"]
        .into_iter()
        .filter_map(|label| {
            let count = counts.get(label)?;
            let plural = if *count == 1 { "" } else { "s" };
            Some(format!("{count} {label}{plural}"))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn plugin_skill_summary(plugin: &PluginPackage) -> String {
    let count = plugin
        .capabilities
        .iter()
        .filter(|capability| capability.kind == RuntimeCapabilityKind::PluginProvidedSkill)
        .count();
    match count {
        0 => "No Skills".to_string(),
        1 => "1 Skill".to_string(),
        count => format!("{count} Skills"),
    }
}
