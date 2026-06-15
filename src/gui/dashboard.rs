use crate::gui::state::{GuiModel, InspectorSection, RenderRow, RenderableView};

pub fn view_name() -> &'static str {
    "Dashboard"
}

pub fn renderable(model: &GuiModel) -> RenderableView {
    let summary = &model.dashboard;
    let main_rows = vec![
        row(
            "agent-space",
            "Agent Space Skills",
            summary.agent_space_instance_count,
        ),
        row(
            "project-agent-space",
            "Project Agent Space Skills",
            summary.project_agent_space_instance_count,
        ),
        RenderRow {
            id: "agents".to_string(),
            cells: vec![
                "Agents".to_string(),
                format!(
                    "{}/{} enabled",
                    summary.enabled_agent_count, summary.agent_count
                ),
            ],
        },
        row("projects", "Recent Projects", summary.recent_project_count),
        row("plugins", "Plugins", model.plugins.len()),
    ];
    let project_lines = if model.project_summaries.is_empty() {
        vec!["No Recent Projects".to_string()]
    } else {
        model
            .project_summaries
            .iter()
            .map(|project| {
                format!(
                    "{} - {} Agent Space Skill(s)",
                    project.name, project.native_skill_count
                )
            })
            .collect()
    };

    RenderableView {
        view: model.active_view,
        title: view_name().to_string(),
        columns: vec!["Metric".to_string(), "Value".to_string()],
        main_rows,
        inspector_sections: vec![
            InspectorSection {
                title: "Project".to_string(),
                lines: vec![
                    selected_project_line(model),
                    format!("Recent Projects {}", summary.recent_project_count),
                ],
            },
            InspectorSection {
                title: "Recent Projects".to_string(),
                lines: project_lines,
            },
            InspectorSection {
                title: "Health".to_string(),
                lines: vec![
                    format!("Registry {}", health_label(&summary.registry_health)),
                    format!("Lock {}", health_label(&summary.lock_health)),
                    format!("Cache {}", health_label(&summary.cache_health)),
                    format!(
                        "Invalid Agent Space toggles {}",
                        summary.invalid_toggle_count
                    ),
                    format!(
                        "Read-only Agent Space instances {}",
                        summary.read_only_count
                    ),
                ],
            },
        ],
        empty_message: None,
    }
}

fn selected_project_line(model: &GuiModel) -> String {
    model
        .scope_project_path()
        .map(|path| format!("Project {path}"))
        .unwrap_or_else(|| "No project selected".to_string())
}

fn health_label(health: &crate::core::status::HealthState) -> &'static str {
    match health {
        crate::core::status::HealthState::Ok => "Ok",
        crate::core::status::HealthState::Warning => "Warning",
        crate::core::status::HealthState::Error => "Error",
    }
}

fn row(id: &str, label: &str, value: usize) -> RenderRow {
    RenderRow {
        id: id.to_string(),
        cells: vec![label.to_string(), value.to_string()],
    }
}
