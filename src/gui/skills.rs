use crate::core::registry::{DeploymentStatus, ManagedSkill, ToggleState};
use crate::gui::state::{
    skill_source_label, GuiModel, GuiRiskReport, InspectorSection, RenderRow, RenderableView,
};

pub fn view_name() -> &'static str {
    "Skills"
}

pub fn renderable(model: &GuiModel) -> RenderableView {
    let main_rows = model
        .skills
        .iter()
        .map(|skill| {
            let deployment_count = model
                .deployments
                .iter()
                .filter(|deployment| deployment.skill_id == skill.id)
                .count();
            RenderRow {
                id: skill.id.to_string(),
                cells: vec![
                    skill.name.clone(),
                    skill_source_label(&skill.source),
                    risk_label(model, skill),
                    deployment_count.to_string(),
                    skill.updated_at.clone(),
                ],
            }
        })
        .collect();

    RenderableView {
        view: model.active_view,
        title: view_name().to_string(),
        columns: vec![
            "Skill".to_string(),
            "Source".to_string(),
            "Risk".to_string(),
            "Project deployments".to_string(),
            "Updated".to_string(),
        ],
        main_rows,
        inspector_sections: inspector_sections(model),
        empty_message: model.skills.is_empty().then_some(
            "No managed Skills yet. Install a local Skill or adopt existing Agent Skills.",
        ),
    }
}

fn inspector_sections(model: &GuiModel) -> Vec<InspectorSection> {
    let Some(skill) = model.selected_skill().or_else(|| model.skills.first()) else {
        return vec![InspectorSection {
            title: "Empty".to_string(),
            lines: vec![
                "No managed Skills yet.".to_string(),
                "Install a local Skill directory, or open Projects to adopt existing Agent Skills."
                    .to_string(),
            ],
        }];
    };
    let source = match &skill.source {
        crate::core::SkillSource::Local { source_path }
        | crate::core::SkillSource::GlobalAgentAdopt { source_path, .. }
        | crate::core::SkillSource::ProjectAdopt { source_path, .. } => source_path.to_string(),
        crate::core::SkillSource::PromotedFromProject { project_path, .. } => {
            project_path.to_string()
        }
    };

    let mut sections = vec![
        InspectorSection {
            title: "Summary".to_string(),
            lines: vec![skill.name.clone(), format!("ID {}", skill.id)],
        },
        InspectorSection {
            title: "Metadata".to_string(),
            lines: metadata_lines(skill),
        },
        InspectorSection {
            title: "Paths".to_string(),
            lines: vec![
                format!("Managed {}", skill.managed_path),
                format!("Source {source}"),
            ],
        },
        InspectorSection {
            title: "Project Deployments".to_string(),
            lines: project_deployment_lines(model, skill),
        },
        InspectorSection {
            title: "Risk Findings".to_string(),
            lines: risk_lines(model, skill),
        },
        InspectorSection {
            title: "Registry Metadata".to_string(),
            lines: vec![
                format!("ID {}", skill.id),
                format!("Hash {}", skill.content_hash),
                format!("Updated {}", skill.updated_at),
            ],
        },
        InspectorSection {
            title: "Actions".to_string(),
            lines: vec![
                "Scan runs the advisory security scanner for this managed Skill.".to_string(),
                "Install copies a local Skill directory into Global Inventory. The source directory is not modified."
                    .to_string(),
                "Deploy copies this managed Skill to the selected project and Agent.".to_string(),
                "Uninstall removes this Skill from Global Inventory. Project copies are not deleted."
                    .to_string(),
            ],
        },
    ];

    sections.retain(|section| !section.lines.is_empty());
    sections
}

fn metadata_lines(skill: &ManagedSkill) -> Vec<String> {
    let Some(metadata) = &skill.metadata else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    if let Some(title) = &metadata.title {
        lines.push(format!("Title {title}"));
    }
    if let Some(description) = &metadata.description {
        lines.push(format!("Description {description}"));
    }
    lines
}

fn project_deployment_lines(model: &GuiModel, skill: &ManagedSkill) -> Vec<String> {
    model
        .deployment_statuses
        .iter()
        .filter(|status| status.record.skill_id == skill.id)
        .map(|status| {
            format!(
                "{} | {} | {} | {}",
                status.record.project_name,
                status.record.agent_id,
                deployment_status_label(status),
                status.record.deployment_path
            )
        })
        .collect()
}

fn risk_label(model: &GuiModel, skill: &ManagedSkill) -> String {
    model
        .skill_risk_report(&skill.id)
        .map(GuiRiskReport::summary_label)
        .unwrap_or_else(|| "Not scanned".to_string())
}

fn risk_lines(model: &GuiModel, skill: &ManagedSkill) -> Vec<String> {
    let Some(report) = model.skill_risk_report(&skill.id) else {
        return vec!["Not scanned yet.".to_string()];
    };

    let mut lines = vec![format!("{}.", report.summary_label())];
    lines.extend(report.findings.iter().map(|finding| {
        format!(
            "{} line {} - {}",
            finding.rule_id,
            finding
                .line
                .map(|line| line.to_string())
                .unwrap_or_else(|| "-".to_string()),
            finding.message
        )
    }));
    lines
}

fn deployment_status_label(status: &DeploymentStatus) -> String {
    if status.missing_managed_source {
        return "Missing managed source".to_string();
    }
    if status.outdated {
        return "Outdated".to_string();
    }
    if status.drift {
        return "Drift".to_string();
    }

    match status.toggle {
        ToggleState::Enabled => "Enabled".to_string(),
        ToggleState::Disabled => "Disabled".to_string(),
        ToggleState::InvalidBothPresent | ToggleState::InvalidBothMissing => "Invalid".to_string(),
    }
}
