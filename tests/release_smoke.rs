use camino::Utf8PathBuf;
use skill_kits::core::{
    adopt::{project_adopt, ProjectAdoptRequest},
    config::{read_config, write_config, RecentProject},
    ids::AgentId,
    install::{install_local_skill, InstallLocalRequest},
    paths::AppPaths,
    project::{
        deploy_project_skill, disable_project_skill, project_deployment_status,
        ProjectDeployRequest, ProjectSkillRequest,
    },
    registry::{read_skills_registry, ToggleState},
    skills::validate_skill_dir,
};
use tempfile::TempDir;

fn utf8(path: impl AsRef<std::path::Path>) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf()).unwrap()
}

#[test]
fn release_smoke_fixture_supports_install_adopt_deploy_and_status_states() {
    let fixture = Utf8PathBuf::from("tests/fixtures/release-smoke");
    let install_source = fixture.join("source-skill");
    let project_source = fixture.join("project/.agents/skills/project-seed");
    validate_skill_dir(&install_source).unwrap();
    validate_skill_dir(&project_source).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let root = utf8(temp_dir.path());
    let paths = AppPaths::from_data_root(root.join("home/.skill-kits"));
    let project = root.join("project");
    std::fs::create_dir_all(&paths.data_root).unwrap();
    std::fs::copy(fixture.join("config.toml"), &paths.config_file).unwrap();
    let mut config = read_config(&paths).unwrap();
    config.recent_projects = vec![RecentProject {
        name: "project".to_string(),
        path: project.clone(),
        last_opened_at: "2026-06-01T00:00:00Z".to_string(),
    }];
    write_config(&paths, &config).unwrap();
    std::fs::create_dir_all(project.join(".agents/skills")).unwrap();
    copy_dir(
        &project_source,
        &project.join(".agents/skills/project-seed"),
    );

    let installed = install_local_skill(
        InstallLocalRequest {
            source_path: &install_source,
        },
        &paths,
    )
    .unwrap();
    assert_eq!(installed.skill.name, "source-skill");
    assert_eq!(
        read_config(&paths).unwrap().recent_projects[0].path,
        project
    );

    let adopt_report = project_adopt(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_name: "project-seed",
    })
    .unwrap();
    assert_eq!(adopt_report.imported, 1);

    let deployed = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "source-skill",
    })
    .unwrap();
    assert_eq!(deployed.record.skill_name, "source-skill");

    disable_project_skill(ProjectSkillRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "source-skill",
    })
    .unwrap();
    let disabled =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "source-skill")
            .unwrap();
    assert_eq!(disabled.toggle, ToggleState::Disabled);

    std::fs::write(
        project.join(".agents/skills/source-skill/local-edit.txt"),
        "release smoke drift\n",
    )
    .unwrap();
    let drifted =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "source-skill")
            .unwrap();
    assert!(drifted.drift);

    let mut skills = read_skills_registry(&paths).unwrap();
    let skill = skills
        .skills
        .iter_mut()
        .find(|skill| skill.name == "source-skill")
        .unwrap();
    skill.content_hash = "release-smoke-new-managed-hash".to_string();
    skill_kits::core::registry::write_skills_registry(&paths, &skills).unwrap();
    let outdated =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "source-skill")
            .unwrap();
    assert!(outdated.outdated);

    skills.skills.retain(|skill| skill.name != "source-skill");
    skill_kits::core::registry::write_skills_registry(&paths, &skills).unwrap();
    let missing =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "source-skill")
            .unwrap();
    assert!(missing.missing_managed_source);
}

fn copy_dir(source: &camino::Utf8Path, target: &camino::Utf8Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = Utf8PathBuf::from_path_buf(entry.path()).unwrap();
        let target_path = target.join(entry.file_name().to_string_lossy().as_ref());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&path, &target_path);
        } else {
            std::fs::copy(&path, target_path).unwrap();
        }
    }
}
