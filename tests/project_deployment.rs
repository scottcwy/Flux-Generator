use camino::{Utf8Path, Utf8PathBuf};
use skill_kits::core::{
    adopt::{project_adopt, project_adopt_all, project_adopt_conflict_as_new, ProjectAdoptRequest},
    agents::{AgentConfig, AgentKind},
    config::Config,
    hash::hash_skill_dir,
    ids::{AgentId, SkillId},
    paths::AppPaths,
    project::{
        deploy_project_skill, disable_project_skill, enable_project_skill,
        project_deployment_status, redeploy_project_skill, remove_project_skill,
        resolve_project_scope, ProjectDeployRequest, ProjectRedeployRequest, ProjectRemoveRequest,
        ProjectSkillRequest, RedeployOutcome,
    },
    registry::{DeploymentsRegistry, ManagedSkill, SkillSource, SkillsRegistry, ToggleState},
    SkillKitsError,
};
use tempfile::TempDir;

fn test_paths(temp_dir: &TempDir) -> AppPaths {
    AppPaths::from_data_root(
        Utf8PathBuf::from_path_buf(temp_dir.path().join(".skill-kits")).unwrap(),
    )
}

fn write_toml<T: serde::Serialize>(path: &Utf8Path, value: &T) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, toml::to_string_pretty(value).unwrap()).unwrap();
}

fn write_skill(path: &Utf8Path, body: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(path.join("SKILL.md"), body).unwrap();
}

fn write_disabled_skill(path: &Utf8Path, body: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(path.join("SKILL.md.disabled"), body).unwrap();
}

fn seed_managed_skill(paths: &AppPaths, id: &str, name: &str, body: &str) -> ManagedSkill {
    let managed_path = paths.skills_dir.join(id);
    write_skill(&managed_path, body);
    let skill = ManagedSkill {
        id: SkillId::new(id),
        name: name.to_string(),
        source: SkillSource::Local {
            source_path: managed_path.clone(),
        },
        managed_path,
        content_hash: String::new(),
        metadata: None,
        created_at: "2026-05-31T00:00:00Z".to_string(),
        updated_at: "2026-05-31T00:00:00Z".to_string(),
    };
    write_toml(
        &paths.skills_registry_file,
        &SkillsRegistry {
            version: 1,
            skills: vec![skill.clone()],
        },
    );
    write_toml(
        &paths.deployments_registry_file,
        &DeploymentsRegistry::default(),
    );
    skill
}

#[test]
fn resolve_project_scope_normalizes_equivalent_existing_paths() {
    let temp_dir = TempDir::new().unwrap();
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    std::fs::create_dir_all(&project).unwrap();
    let project_dot = project.join(".");
    let project_link = Utf8PathBuf::from_path_buf(temp_dir.path().join("project-link")).unwrap();
    std::os::unix::fs::symlink(&project, &project_link).unwrap();

    let direct = resolve_project_scope(Some(&project)).unwrap();
    let dotted = resolve_project_scope(Some(&project_dot)).unwrap();
    let linked = resolve_project_scope(Some(&project_link)).unwrap();

    assert_eq!(direct.name, "project");
    assert_eq!(dotted.name, "project");
    assert_eq!(linked.name, "project");
    assert_eq!(direct.path, dotted.path);
    assert_eq!(direct.path, linked.path);
}

#[test]
fn configured_custom_agent_project_skill_dir_is_used_for_deploy_status_and_adopt() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let custom_agent = AgentConfig {
        id: AgentId::new("custom"),
        label: "Custom".to_string(),
        kind: AgentKind::Custom,
        global_skill_dirs: Vec::new(),
        project_skill_dirs: vec![".custom/skills".into()],
        enabled: true,
    };
    write_toml(
        &paths.config_file,
        &Config {
            agents: vec![custom_agent],
            ..Config::default()
        },
    );
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();

    let deployed = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("custom"),
        skill_query: "frontend-design",
    })
    .unwrap();

    assert_eq!(
        deployed.record.deployment_path,
        project.join(".custom/skills/frontend-design")
    );
    assert!(project
        .join(".custom/skills/frontend-design/SKILL.md")
        .exists());
    let status =
        project_deployment_status(&paths, &project, &AgentId::new("custom"), "frontend-design")
            .unwrap();
    assert_eq!(
        status.record.deployment_path,
        deployed.record.deployment_path
    );

    write_toml(&paths.skills_registry_file, &SkillsRegistry::default());
    write_toml(
        &paths.deployments_registry_file,
        &DeploymentsRegistry::default(),
    );
    write_skill(&project.join(".custom/skills/local-only"), "# Local only\n");

    let report = project_adopt(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("custom"),
        skill_name: "local-only",
    })
    .unwrap();

    assert_eq!(report.imported, 1);
    let adopted =
        project_deployment_status(&paths, &project, &AgentId::new("custom"), "local-only").unwrap();
    assert_eq!(
        adopted.record.deployment_path,
        project.join(".custom/skills/local-only")
    );
}

#[test]
fn project_deploy_creates_enabled_skill_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let managed = seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    std::fs::write(managed.managed_path.join("notes.txt"), "keep me").unwrap();
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();

    let status = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();

    assert_eq!(status.toggle, ToggleState::Enabled);
    assert!(project
        .join(".agents/skills/frontend-design/SKILL.md")
        .exists());
}

#[test]
fn deploy_records_managed_hash_without_toggle_normalization() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let managed_path = paths.skills_dir.join("disabled-source-a1b2c3d4");
    write_disabled_skill(&managed_path, "# Disabled source\n");
    let skill = ManagedSkill {
        id: SkillId::new("disabled-source-a1b2c3d4"),
        name: "disabled-source".to_string(),
        source: SkillSource::Local {
            source_path: managed_path.clone(),
        },
        managed_path,
        content_hash: String::new(),
        metadata: None,
        created_at: "2026-05-31T00:00:00Z".to_string(),
        updated_at: "2026-05-31T00:00:00Z".to_string(),
    };
    write_toml(
        &paths.skills_registry_file,
        &SkillsRegistry {
            version: 1,
            skills: vec![skill],
        },
    );
    write_toml(
        &paths.deployments_registry_file,
        &DeploymentsRegistry::default(),
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();

    let status = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "disabled-source",
    })
    .unwrap();

    assert!(project
        .join(".agents/skills/disabled-source/SKILL.md")
        .exists());
    assert!(!project
        .join(".agents/skills/disabled-source/SKILL.md.disabled")
        .exists());
    assert_ne!(
        status.record.deployed_from_hash, status.record.baseline_hash,
        "Managed hashes use literal file paths; only project drift hashes normalize toggles"
    );
}

#[test]
fn enable_disable_only_renames_toggle_file() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let managed = seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    std::fs::write(managed.managed_path.join("notes.txt"), "keep me").unwrap();
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    let deployment = project.join(".agents/skills/frontend-design");

    disable_project_skill(ProjectSkillRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();

    assert!(!deployment.join("SKILL.md").exists());
    assert!(deployment.join("SKILL.md.disabled").exists());
    assert_eq!(
        std::fs::read_to_string(deployment.join("notes.txt")).unwrap(),
        "keep me"
    );
    let disabled_status =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "frontend-design")
            .unwrap();
    assert_eq!(disabled_status.toggle, ToggleState::Disabled);
    assert!(!disabled_status.drift);

    enable_project_skill(ProjectSkillRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();

    assert!(deployment.join("SKILL.md").exists());
    assert!(!deployment.join("SKILL.md.disabled").exists());
    assert_eq!(
        std::fs::read_to_string(deployment.join("notes.txt")).unwrap(),
        "keep me"
    );
}

#[test]
fn deploy_blocks_on_unmanaged_same_name_target() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    write_skill(
        &project.join(".agents/skills/frontend-design"),
        "# Existing\n",
    );

    let err = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap_err();

    assert!(matches!(err, SkillKitsError::DeployConflict { .. }));
}

#[test]
fn project_adopt_records_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    write_toml(&paths.skills_registry_file, &SkillsRegistry::default());
    write_toml(
        &paths.deployments_registry_file,
        &DeploymentsRegistry::default(),
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    write_skill(
        &project.join(".agents/skills/frontend-design"),
        "# Project copy\n",
    );

    let report = project_adopt(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_name: "frontend-design",
    })
    .unwrap();

    assert_eq!(report.imported, 1);
    let status =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "frontend-design")
            .unwrap();
    assert!(!status.record.baseline_hash.is_empty());
    assert_eq!(status.record.baseline_hash, status.current_hash.unwrap());
}

#[test]
fn project_adopt_records_skill_and_deployment_consistently() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    write_toml(&paths.skills_registry_file, &SkillsRegistry::default());
    write_toml(
        &paths.deployments_registry_file,
        &DeploymentsRegistry::default(),
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    let deployment_dir = project.join(".agents/skills/frontend-design");
    write_skill(&deployment_dir, "# Project copy\n");

    project_adopt(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_name: "frontend-design",
    })
    .unwrap();

    let skills: SkillsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.skills_registry_file).unwrap()).unwrap();
    let deployments: DeploymentsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.deployments_registry_file).unwrap())
            .unwrap();
    assert_eq!(skills.skills.len(), 1);
    assert_eq!(deployments.deployments.len(), 1);
    let skill = &skills.skills[0];
    let deployment = &deployments.deployments[0];
    assert_eq!(deployment.skill_id, skill.id);
    assert_eq!(deployment.deployed_from_hash, skill.content_hash);
    assert_eq!(
        deployment.baseline_hash,
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "frontend-design")
            .unwrap()
            .current_hash
            .unwrap()
    );
    assert_eq!(
        skill.content_hash,
        hash_skill_dir(&skill.managed_path).unwrap()
    );
}

#[test]
fn project_adopt_all_reports_partial_success() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "existing-a1b2c3d4",
        "existing",
        "# Managed existing\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    write_skill(&project.join(".agents/skills/fresh"), "# Fresh\n");
    write_skill(
        &project.join(".agents/skills/existing"),
        "# Project conflict\n",
    );

    let report = project_adopt_all(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_name: "",
    })
    .unwrap();

    assert_eq!(report.imported, 1);
    assert_eq!(report.conflicts, 1);
    assert!(project_deployment_status(&paths, &project, &AgentId::new("codex"), "fresh").is_ok());
}

#[test]
fn project_adopt_conflict_as_new_imports_same_name_different_hash_without_replacing_original() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let mut original = seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Managed original\n",
    );
    original.content_hash = hash_skill_dir(&original.managed_path).unwrap();
    write_toml(
        &paths.skills_registry_file,
        &SkillsRegistry {
            version: 1,
            skills: vec![original.clone()],
        },
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    let deployment_dir = project.join(".agents/skills/frontend-design");
    write_skill(&deployment_dir, "# Project conflict\n");
    let original_body = std::fs::read_to_string(original.managed_path.join("SKILL.md")).unwrap();

    let report = project_adopt_conflict_as_new(ProjectAdoptRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_name: "frontend-design",
    })
    .unwrap();

    assert_eq!(report.imported, 1);
    assert_eq!(report.conflicts, 0);
    assert_eq!(
        std::fs::read_to_string(original.managed_path.join("SKILL.md")).unwrap(),
        original_body
    );

    let skills: SkillsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.skills_registry_file).unwrap()).unwrap();
    assert_eq!(skills.skills.len(), 2);
    let adopted = skills
        .skills
        .iter()
        .find(|skill| skill.id != original.id)
        .unwrap();
    assert_eq!(adopted.name, "frontend-design");
    assert_ne!(adopted.id, original.id);
    assert_ne!(adopted.managed_path, original.managed_path);
    assert_eq!(
        std::fs::read_to_string(adopted.managed_path.join("SKILL.md")).unwrap(),
        "# Project conflict\n"
    );
    assert_eq!(
        adopted.content_hash,
        hash_skill_dir(&adopted.managed_path).unwrap()
    );

    let deployments: DeploymentsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.deployments_registry_file).unwrap())
            .unwrap();
    assert_eq!(deployments.deployments.len(), 1);
    let deployment = &deployments.deployments[0];
    assert_eq!(deployment.skill_id, adopted.id);
    assert_eq!(deployment.deployment_path, deployment_dir);
    assert_eq!(deployment.skill_name, "frontend-design");
    assert_eq!(deployment.deployed_from_hash, adopted.content_hash);
    assert_eq!(
        deployment.baseline_hash,
        hash_skill_dir(&deployment_dir).unwrap()
    );
}

#[test]
fn redeploy_blocks_on_drift() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    std::fs::write(
        project.join(".agents/skills/frontend-design/SKILL.md"),
        "# Local edit\n",
    )
    .unwrap();

    let err = redeploy_project_skill(ProjectRedeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
        overwrite: false,
        promote: false,
    })
    .unwrap_err();

    assert!(matches!(err, SkillKitsError::DeploymentDrift { .. }));
}

#[test]
fn repeated_project_deploy_blocks_on_existing_drift() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    std::fs::write(
        project.join(".agents/skills/frontend-design/SKILL.md"),
        "# Project edit\n",
    )
    .unwrap();

    let err = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap_err();

    assert!(matches!(err, SkillKitsError::DeploymentDrift { .. }));
    assert_eq!(
        std::fs::read_to_string(project.join(".agents/skills/frontend-design/SKILL.md")).unwrap(),
        "# Project edit\n"
    );
}

#[test]
fn promote_creates_managed_skill_fork() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    std::fs::write(
        project.join(".agents/skills/frontend-design/SKILL.md"),
        "# Local fork\n",
    )
    .unwrap();

    let outcome = redeploy_project_skill(ProjectRedeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
        overwrite: false,
        promote: true,
    })
    .unwrap();

    let RedeployOutcome::Promoted(fork) = outcome else {
        panic!("expected promoted fork");
    };
    assert_ne!(fork.id.as_str(), "frontend-design-a1b2c3d4");
    assert_eq!(
        std::fs::read_to_string(fork.managed_path.join("SKILL.md")).unwrap(),
        "# Local fork\n"
    );
}

#[test]
fn promote_relinks_deployment_to_fork_consistently() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    let deployment_dir = project.join(".agents/skills/frontend-design");
    std::fs::write(deployment_dir.join("SKILL.md"), "# Local fork\n").unwrap();

    let outcome = redeploy_project_skill(ProjectRedeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
        overwrite: false,
        promote: true,
    })
    .unwrap();

    let RedeployOutcome::Promoted(fork) = outcome else {
        panic!("expected promoted fork");
    };
    let skills: SkillsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.skills_registry_file).unwrap()).unwrap();
    let deployments: DeploymentsRegistry =
        toml::from_str(&std::fs::read_to_string(&paths.deployments_registry_file).unwrap())
            .unwrap();
    let deployment = deployments
        .deployments
        .iter()
        .find(|deployment| deployment.skill_id == fork.id)
        .expect("deployment relinked to promoted fork");
    let stored_fork = skills
        .skills
        .iter()
        .find(|skill| skill.id == fork.id)
        .expect("promoted fork recorded in skills registry");
    assert_eq!(deployment.deployed_from_hash, stored_fork.content_hash);
    assert_eq!(
        deployment.baseline_hash,
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "frontend-design")
            .unwrap()
            .current_hash
            .unwrap()
    );
    assert_eq!(
        stored_fork.content_hash,
        hash_skill_dir(&stored_fork.managed_path).unwrap()
    );
}

#[test]
fn remove_deletes_only_selected_deployment_dir() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    write_skill(&project.join(".agents/skills/other-skill"), "# Other\n");

    remove_project_skill(ProjectRemoveRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
        force: true,
    })
    .unwrap();

    assert!(!project.join(".agents/skills/frontend-design").exists());
    assert!(project.join(".agents/skills/other-skill/SKILL.md").exists());
    assert!(project.join(".agents/skills").exists());
}

#[test]
fn remove_missing_managed_source_without_drift_does_not_require_force() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();
    write_toml(&paths.skills_registry_file, &SkillsRegistry::default());

    remove_project_skill(ProjectRemoveRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
        force: false,
    })
    .unwrap();

    assert!(!project.join(".agents/skills/frontend-design").exists());
}

#[test]
fn status_detects_outdated_invalid_toggle_and_missing_source() {
    let temp_dir = TempDir::new().unwrap();
    let paths = test_paths(&temp_dir);
    let mut skill = seed_managed_skill(
        &paths,
        "frontend-design-a1b2c3d4",
        "frontend-design",
        "# Frontend\n",
    );
    let project = Utf8PathBuf::from_path_buf(temp_dir.path().join("project")).unwrap();
    let deployed = deploy_project_skill(ProjectDeployRequest {
        app_paths: &paths,
        project_path: &project,
        agent_id: &AgentId::new("codex"),
        skill_query: "frontend-design",
    })
    .unwrap();

    skill.content_hash = "new-managed-hash".to_string();
    write_toml(
        &paths.skills_registry_file,
        &SkillsRegistry {
            version: 1,
            skills: vec![skill],
        },
    );
    std::fs::write(
        project.join(".agents/skills/frontend-design/SKILL.md.disabled"),
        "# Disabled too\n",
    )
    .unwrap();

    let status =
        project_deployment_status(&paths, &project, &AgentId::new("codex"), "frontend-design")
            .unwrap();

    assert!(status.outdated);
    assert_eq!(status.toggle, ToggleState::InvalidBothPresent);

    write_toml(&paths.skills_registry_file, &SkillsRegistry::default());
    let missing = project_deployment_status(
        &paths,
        &project,
        &AgentId::new("codex"),
        &deployed.record.skill_name,
    )
    .unwrap();

    assert!(missing.missing_managed_source);
}
