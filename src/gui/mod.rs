pub mod agents;
pub mod dashboard;
pub mod icons;
pub mod plugins;
pub mod projects;
pub mod skills;
pub mod state;

use crate::core::paths::AppPaths;
use crate::core::ToggleState;
use camino::Utf8Path;
use eframe::egui;
use state::{
    AgentEditorMode, GuiController, GuiModel, GuiScope, NavigationView, RenderRow, RenderableView,
    UiColors, DRIFT_REMOVE_CONFIRMATION_MESSAGE, SKILL_INSTANCE_DISABLE_CONFIRMATION_MESSAGE,
};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

const MAIN_ROW_HEIGHT: f32 = 34.0;
const MAIN_CONTENT_INSET: f32 = 12.0;
const TABLE_COLUMN_GAP: f32 = 8.0;
const STATUS_BADGE_HEIGHT: f32 = 22.0;
const STATUS_BADGE_HORIZONTAL_PADDING: f32 = 7.0;
const STATUS_BADGE_ICON_WIDTH: f32 = 14.0;
const STATUS_BADGE_ICON_TEXT_GAP: f32 = 6.0;
const STATUS_BADGE_TEXT_CHAR_WIDTH: f32 = 7.0;
const DASHBOARD_VALUE_WIDTH: f32 = 132.0;
pub const SIDEBAR_WIDTH: f32 = 244.0;
pub const SIDEBAR_NAV_ROW_HEIGHT: f32 = 36.0;
const SIDEBAR_SCOPE_ROW_HEIGHT: f32 = 32.0;
const SIDEBAR_ROW_OUTER_INSET: f32 = 8.0;
const SIDEBAR_ROW_CONTENT_INSET: f32 = 18.0;
const SIDEBAR_ICON_COLUMN_WIDTH: f32 = 28.0;
const SIDEBAR_ROW_RADIUS: f32 = 5.0;
const MACOS_CHROME_TOP_INSET: f32 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarGridMetrics {
    pub row_outer_inset: f32,
    pub icon_x: f32,
    pub label_x: f32,
    pub section_label_x: f32,
    pub row_radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashboardOverviewGrid {
    pub heading_x: f32,
    pub divider_start_x: f32,
    pub divider_end_x: f32,
    pub row_label_x: f32,
    pub label_width: f32,
    pub value_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchContentGrid {
    pub inset: f32,
    pub left: f32,
    pub right: f32,
    pub width: f32,
    pub row_rounding: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchButtonMetrics {
    pub height: f32,
    pub radius: f32,
    pub icon_width: f32,
    pub horizontal_padding: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchCommandRowMetrics {
    pub height: f32,
    pub radius: f32,
    pub inset: f32,
    pub icon_x: f32,
    pub label_x: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchTableMetrics {
    pub inset: f32,
    pub column_gap: f32,
    pub column_widths: Vec<f32>,
    pub column_lefts: Vec<f32>,
    pub content_width: f32,
    pub table_width: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillAction {
    ScanAgentSpaces,
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginAction {
    BackToPlugins,
    ScanPlugins,
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectAction {
    Refresh,
    AdoptSelected,
    AdoptAll,
    ImportAsNew,
    Skip,
    Deploy,
    Enable,
    Disable,
    Redeploy,
    Overwrite,
    Promote,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentAction {
    EditSelected,
    ResetDefault,
    RemoveCustom,
    AddCustom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkbenchCellStyle {
    Text,
    Mono,
    StatusBadge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkbenchCellAlignment {
    Center,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathFieldKind {
    ExistingDirectory,
    AgentProjectDir,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorLinePresentation {
    pub label: String,
    pub value: String,
    pub kind: InspectorLineKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorLineKind {
    Text,
    Mono,
    Path,
    StatusBadge,
}

impl AgentAction {
    const EMPTY: [Self; 1] = [Self::AddCustom];

    fn label(self) -> &'static str {
        match self {
            Self::EditSelected => "Edit path",
            Self::ResetDefault => "Reset default",
            Self::RemoveCustom => "Remove custom",
            Self::AddCustom => "Add custom",
        }
    }
}

impl ProjectAction {
    const REFRESH: [Self; 1] = [Self::Refresh];

    fn label(self) -> &'static str {
        match self {
            Self::Refresh => "Refresh",
            Self::AdoptSelected => "Adopt selected",
            Self::AdoptAll => "Adopt all",
            Self::ImportAsNew => "Import as new",
            Self::Skip => "Skip",
            Self::Deploy => "Deploy",
            Self::Enable => "Enable",
            Self::Disable => "Disable",
            Self::Redeploy => "Redeploy",
            Self::Overwrite => "Overwrite",
            Self::Promote => "Promote",
            Self::Remove => "Remove",
        }
    }
}

impl SkillAction {
    fn label(self) -> &'static str {
        match self {
            Self::ScanAgentSpaces => "Scan Agent Spaces",
            Self::Enable => "Enable",
            Self::Disable => "Disable",
        }
    }
}

impl PluginAction {
    fn label(self) -> &'static str {
        match self {
            Self::BackToPlugins => "Back to Plugins",
            Self::ScanPlugins => "Scan Plugins",
            Self::Enable => "Enable Plugin",
            Self::Disable => "Disable Plugin",
        }
    }
}

pub fn skill_action_command_label(action: SkillAction, confirming_disable: bool) -> &'static str {
    if matches!(action, SkillAction::Disable) && confirming_disable {
        "Confirm Disable"
    } else {
        action.label()
    }
}

pub fn skill_actions(model: &GuiModel) -> Vec<SkillAction> {
    let mut actions = vec![SkillAction::ScanAgentSpaces];
    if model.selected_skill_instance().is_none() {
        return actions;
    }

    if let Some(instance) = model.selected_skill_instance() {
        if instance.writable {
            match instance.toggle_state {
                ToggleState::Enabled => actions.push(SkillAction::Disable),
                ToggleState::Disabled => actions.push(SkillAction::Enable),
                ToggleState::InvalidBothPresent | ToggleState::InvalidBothMissing => {}
            }
        }
    }
    actions
}

pub fn agent_actions(model: &GuiModel) -> Vec<AgentAction> {
    let Some(agent) = model.selected_agent() else {
        return AgentAction::EMPTY.to_vec();
    };

    let mut actions = vec![AgentAction::EditSelected];
    match agent.kind {
        crate::core::agents::AgentKind::BuiltIn => actions.push(AgentAction::ResetDefault),
        crate::core::agents::AgentKind::Custom => actions.push(AgentAction::RemoveCustom),
    }
    actions.push(AgentAction::AddCustom);
    actions
}

pub fn project_actions(model: &GuiModel) -> Vec<ProjectAction> {
    let mut actions = ProjectAction::REFRESH.to_vec();
    if let Some(instance) = model.selected_skill_instance() {
        if instance.writable {
            match instance.toggle_state {
                ToggleState::Enabled => actions.push(ProjectAction::Disable),
                ToggleState::Disabled => actions.push(ProjectAction::Enable),
                ToggleState::InvalidBothPresent | ToggleState::InvalidBothMissing => {}
            }
        }
    }
    actions
}

pub fn plugin_actions(model: &GuiModel) -> Vec<PluginAction> {
    let mut actions = Vec::new();
    if model.selected_plugin().is_some() {
        actions.push(PluginAction::BackToPlugins);
    }
    actions.push(PluginAction::ScanPlugins);
    let Some(plugin) = model.selected_plugin() else {
        return actions;
    };
    if plugin.can_toggle {
        match plugin.status {
            crate::core::plugins::PluginStatus::Enabled => actions.push(PluginAction::Disable),
            crate::core::plugins::PluginStatus::Disabled => actions.push(PluginAction::Enable),
            crate::core::plugins::PluginStatus::Unknown
            | crate::core::plugins::PluginStatus::Invalid => {}
        }
    }
    actions
}

pub fn workbench_cell_style(column: &str) -> WorkbenchCellStyle {
    match column {
        "Status" | "Toggle" | "Validation" | "Enabled" | "Writable" | "Managed" | "Outdated"
        | "Drift" | "Risk" | "Read-only" => WorkbenchCellStyle::StatusBadge,
        "Path"
        | "Project skill directories"
        | "Skill ID"
        | "Instance ID"
        | "Plugin ID"
        | "Plugin Key"
        | "Capability ID"
        | "Hash"
        | "Command" => WorkbenchCellStyle::Mono,
        _ => WorkbenchCellStyle::Text,
    }
}

pub fn workbench_cell_alignment(_column: &str) -> WorkbenchCellAlignment {
    WorkbenchCellAlignment::Center
}

pub fn workbench_cell_content_offset_x(column: &str) -> f32 {
    match column {
        "Status" => 8.0,
        _ => 0.0,
    }
}

pub fn workbench_status_badge_rect(cell_rect: egui::Rect, value: &str) -> egui::Rect {
    let natural_width = (STATUS_BADGE_HORIZONTAL_PADDING * 2.0)
        + STATUS_BADGE_ICON_WIDTH
        + STATUS_BADGE_ICON_TEXT_GAP
        + (value.chars().count() as f32 * STATUS_BADGE_TEXT_CHAR_WIDTH);
    let width = natural_width.min(cell_rect.width());
    egui::Rect::from_center_size(cell_rect.center(), egui::vec2(width, STATUS_BADGE_HEIGHT))
}

pub fn workbench_row_accepts_keyboard_key(key: egui::Key) -> bool {
    matches!(key, egui::Key::Enter | egui::Key::Space)
}

pub fn workbench_chrome_top_inset() -> f32 {
    if cfg!(target_os = "macos") {
        MACOS_CHROME_TOP_INSET
    } else {
        0.0
    }
}

pub fn workbench_renders_inspector_panel() -> bool {
    false
}

pub fn sidebar_nav_label(view: NavigationView) -> String {
    icons::button_label(icons::navigation_icon(view), view.title())
}

pub fn sidebar_grid_metrics() -> SidebarGridMetrics {
    let label_x = SIDEBAR_ROW_CONTENT_INSET + SIDEBAR_ICON_COLUMN_WIDTH;
    SidebarGridMetrics {
        row_outer_inset: SIDEBAR_ROW_OUTER_INSET,
        icon_x: SIDEBAR_ROW_CONTENT_INSET,
        label_x,
        section_label_x: label_x,
        row_radius: SIDEBAR_ROW_RADIUS,
    }
}

pub fn dashboard_overview_grid(available_width: f32) -> DashboardOverviewGrid {
    let content_grid = workbench_content_grid(available_width);
    let label_width = (content_grid.width - DASHBOARD_VALUE_WIDTH).max(160.0);
    DashboardOverviewGrid {
        heading_x: content_grid.left,
        divider_start_x: content_grid.left,
        divider_end_x: content_grid.right,
        row_label_x: content_grid.left,
        label_width,
        value_width: DASHBOARD_VALUE_WIDTH,
    }
}

pub fn workbench_content_grid(available_width: f32) -> WorkbenchContentGrid {
    let outer_width = available_width.max(360.0);
    let left = MAIN_CONTENT_INSET;
    let right = outer_width - MAIN_CONTENT_INSET;
    WorkbenchContentGrid {
        inset: MAIN_CONTENT_INSET,
        left,
        right,
        width: right - left,
        row_rounding: 5.0,
    }
}

pub fn workbench_button_metrics() -> WorkbenchButtonMetrics {
    WorkbenchButtonMetrics {
        height: 28.0,
        radius: 4.0,
        icon_width: 18.0,
        horizontal_padding: 9.0,
    }
}

pub fn workbench_command_row_metrics() -> WorkbenchCommandRowMetrics {
    WorkbenchCommandRowMetrics {
        height: SIDEBAR_SCOPE_ROW_HEIGHT,
        radius: workbench_content_grid(620.0).row_rounding,
        inset: MAIN_CONTENT_INSET,
        icon_x: 10.0,
        label_x: 36.0,
    }
}

pub fn workbench_button_label(icon: &str, label: &str) -> String {
    icons::button_label(icon, label)
}

pub fn workbench_button_hover_fill(
    enabled: bool,
    hovered: bool,
    colors: UiColors,
) -> egui::Color32 {
    if enabled {
        workbench_row_fill(false, hovered, colors)
    } else {
        egui::Color32::TRANSPARENT
    }
}

pub fn workbench_static_labels_selectable() -> bool {
    false
}

pub fn workbench_hover_tooltips_enabled() -> bool {
    false
}

pub fn workbench_filter_width(label: &str) -> f32 {
    match label {
        "Agent" => 136.0,
        "Scope" => 116.0,
        "Status" => 112.0,
        _ => 120.0,
    }
}

pub fn plugin_row_disclosure(view: NavigationView, columns: &[String]) -> Option<&'static str> {
    if !matches!(view, NavigationView::Plugins) {
        return None;
    }
    columns
        .first()
        .is_some_and(|column| column == "Plugin")
        .then_some(icons::CHEVRON_RIGHT)
}

pub fn workbench_table_metrics(columns: &[String], viewport_width: f32) -> WorkbenchTableMetrics {
    let mut column_widths = table_column_widths(columns);
    let natural_content_width = table_content_width(&column_widths);
    let natural_table_width = natural_content_width + (MAIN_CONTENT_INSET * 2.0);
    let table_width = natural_table_width.max(viewport_width);
    if stretches_columns_to_viewport(columns) && table_width > natural_table_width {
        let total_gap = TABLE_COLUMN_GAP * columns.len().saturating_sub(1) as f32;
        let equal_width =
            (table_width - (MAIN_CONTENT_INSET * 2.0) - total_gap) / columns.len().max(1) as f32;
        column_widths = vec![equal_width; columns.len()];
    }
    let content_width = table_content_width(&column_widths);
    let mut cursor = MAIN_CONTENT_INSET;
    let column_lefts = column_widths
        .iter()
        .map(|width| {
            let left = cursor;
            cursor += *width + TABLE_COLUMN_GAP;
            left
        })
        .collect();

    WorkbenchTableMetrics {
        inset: MAIN_CONTENT_INSET,
        column_gap: TABLE_COLUMN_GAP,
        column_widths,
        column_lefts,
        content_width,
        table_width,
    }
}

fn table_content_width(column_widths: &[f32]) -> f32 {
    column_widths.iter().sum::<f32>()
        + TABLE_COLUMN_GAP * column_widths.len().saturating_sub(1) as f32
}

fn is_agent_table_columns(columns: &[String]) -> bool {
    columns
        == [
            "Agent".to_string(),
            "Project skill directories".to_string(),
            "Enabled".to_string(),
            "Validation".to_string(),
        ]
}

fn is_skill_table_columns(columns: &[String]) -> bool {
    columns
        == [
            "Skill".to_string(),
            "Agent".to_string(),
            "Scope".to_string(),
            "Status".to_string(),
            "Source".to_string(),
        ]
}

fn stretches_columns_to_viewport(columns: &[String]) -> bool {
    is_agent_table_columns(columns) || is_skill_table_columns(columns)
}

pub fn workbench_row_fill(selected: bool, hovered: bool, colors: UiColors) -> egui::Color32 {
    if selected {
        colors.surface_3
    } else if hovered {
        colors.surface_2
    } else {
        egui::Color32::TRANSPARENT
    }
}

pub fn status_badge_fill(
    value: &str,
    row_hovered: bool,
    selected: bool,
    colors: UiColors,
) -> egui::Color32 {
    let _ = value;
    let _ = row_hovered;
    let _ = selected;
    let _ = colors;
    egui::Color32::TRANSPARENT
}

pub fn status_badge_stroke(
    _value: &str,
    row_hovered: bool,
    selected: bool,
    colors: UiColors,
) -> egui::Stroke {
    let _ = row_hovered;
    let _ = selected;
    let _ = colors;
    egui::Stroke::NONE
}

pub fn path_validation_message(value: &str, kind: PathFieldKind) -> Option<&'static str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some("Choose a folder.");
    }

    match kind {
        PathFieldKind::ExistingDirectory => {
            if Utf8Path::new(trimmed).is_dir() {
                None
            } else {
                Some("Folder does not exist.")
            }
        }
        PathFieldKind::AgentProjectDir => {
            if Utf8Path::new(trimmed).is_absolute() {
                Some("Use a project-relative directory.")
            } else {
                None
            }
        }
    }
}

pub fn inspector_line_presentation(line: &str) -> InspectorLinePresentation {
    const LABELS: &[&str] = &[
        "Missing managed source",
        "Project dir",
        "Project path",
        "Project Agent Space instances",
        "Agent Space instances",
        "Instance ID",
        "Agent id",
        "Skill dir",
        "Enabled",
        "Disabled",
        "Content hash",
        "Scanned hash",
        "Registry",
        "Status",
        "Source",
        "Writable",
        "Managed",
        "Agent",
        "Scope",
        "Kind",
        "Title",
        "Description",
        "Plugin id",
        "Plugin key",
        "Capability id",
        "Config path",
        "Package path",
        "Manifest path",
        "Lock",
        "Cache",
    ];

    for label in LABELS {
        if let Some(value) = line.strip_prefix(&format!("{label} ")) {
            return InspectorLinePresentation {
                label: (*label).to_string(),
                value: value.to_string(),
                kind: inspector_line_kind(label, value),
            };
        }
    }

    InspectorLinePresentation {
        label: String::new(),
        value: line.to_string(),
        kind: InspectorLineKind::Text,
    }
}

fn inspector_line_kind(label: &str, value: &str) -> InspectorLineKind {
    match label {
        "Skill dir" | "Enabled" | "Disabled" | "Project dir" | "Project path" | "Config path"
        | "Package path" | "Manifest path" => InspectorLineKind::Path,
        "Instance ID" | "Agent id" | "Content hash" | "Scanned hash" | "Plugin id"
        | "Plugin key" | "Capability id" => InspectorLineKind::Mono,
        "Status"
        | "Writable"
        | "Managed"
        | "Registry"
        | "Lock"
        | "Cache"
        | "Missing managed source" => InspectorLineKind::StatusBadge,
        _ if value.starts_with('/') || value.starts_with("~/") => InspectorLineKind::Path,
        _ => InspectorLineKind::Text,
    }
}

pub struct SkillKitsGuiApp {
    model: GuiModel,
    controller: GuiController,
    colors: UiColors,
    running_intent: Option<RunningIntent>,
}

struct RunningIntent {
    label: &'static str,
    intent: state::GuiActionIntent,
    receiver: Receiver<GuiModel>,
}

impl SkillKitsGuiApp {
    pub fn from_paths(paths: &AppPaths) -> crate::core::Result<Self> {
        Ok(Self::new(
            GuiModel::load(paths)?,
            GuiController::new(paths.clone()),
        ))
    }

    pub fn new(model: GuiModel, controller: GuiController) -> Self {
        Self {
            model,
            controller,
            colors: UiColors::dark(),
            running_intent: None,
        }
    }

    pub fn model(&self) -> &GuiModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut GuiModel {
        &mut self.model
    }

    pub fn has_running_intent(&self) -> bool {
        self.running_intent.is_some()
    }

    pub fn action_status_label(&self) -> String {
        if let Some(running) = &self.running_intent {
            return format!(
                "Working: {} ({} queued)",
                running.label,
                self.model.pending_intents().len()
            );
        }
        self.model.pending_action_status_label()
    }
}

pub fn run_native(paths: AppPaths) -> anyhow::Result<()> {
    eframe::run_native(
        "Skill-kits",
        native_options(),
        Box::new(move |cc| {
            icons::install_font(&cc.egui_ctx);
            Ok(Box::new(SkillKitsGuiApp::from_paths(&paths)?))
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([960.0, 620.0])
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_titlebar_buttons_shown(true),
        ..Default::default()
    }
}

impl eframe::App for SkillKitsGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_dark_theme(ctx, self.colors);
        if self.collect_finished_intent() {
            ctx.request_repaint();
        }
        self.dispatch_one_pending_intent();
        if self.has_running_intent() {
            ctx.request_repaint();
        }

        let chrome_top_inset = workbench_chrome_top_inset();
        if chrome_top_inset > 0.0 {
            egui::TopBottomPanel::top("macos_chrome_spacer")
                .frame(egui::Frame::none().fill(self.colors.surface_1))
                .show_separator_line(false)
                .exact_height(chrome_top_inset)
                .show(ctx, |ui| {
                    render_sidebar_boundary_extension(ui);
                });
        }

        egui::SidePanel::left("sidebar")
            .frame(egui::Frame::none().fill(self.colors.surface_1))
            .resizable(false)
            .exact_width(SIDEBAR_WIDTH)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                for view in NavigationView::ORDER {
                    let selected = self.model.active_view == view;
                    if render_sidebar_nav_item(ui, view, selected, self.colors).clicked() {
                        self.model.navigate(view);
                    }
                }
                ui.add_space(12.0);
                ui.add_space(4.0);
                render_sidebar_section_label(ui, "Scope", self.colors);
                if render_sidebar_scope_item(
                    ui,
                    icons::ASSET,
                    "Managed Inventory",
                    matches!(self.model.active_scope, GuiScope::GlobalInventory),
                    self.colors,
                )
                .clicked()
                {
                    self.model.select_scope(GuiScope::GlobalInventory);
                }
                ui.add_space(4.0);
                render_sidebar_section_label(ui, "Recent Projects", self.colors);
                let projects = self.model.recent_projects.clone();
                egui::ScrollArea::vertical()
                    .id_salt("sidebar_recent_projects")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for project in projects {
                            let selected = matches!(
                                &self.model.active_scope,
                                GuiScope::Project(path) if path == &project.path
                            );
                            let response = render_sidebar_scope_item(
                                ui,
                                icons::PROJECT,
                                &project.name,
                                selected,
                                self.colors,
                            );
                            if response.clicked() {
                                self.model.select_scope(GuiScope::Project(project.path));
                            }
                        }
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.colors.canvas))
            .show(ctx, |ui| {
                let renderable = self.model.renderable_view();
                render_main(ui, &mut self.model, &renderable, self.colors);
            });
    }
}

impl SkillKitsGuiApp {
    pub fn execute_one_pending_intent(&mut self) {
        let _ = self.model.execute_next_intent(&self.controller);
    }

    pub fn dispatch_one_pending_intent(&mut self) {
        if self.running_intent.is_some() || self.model.pending_intents().is_empty() {
            return;
        }
        let label = self.model.next_action_label().unwrap_or("Action");
        let intent = self.model.pending_intents()[0].clone();
        let mut model = self.model.clone();
        let controller = self.controller.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let _ = model.execute_next_intent(&controller);
            let _ = sender.send(model);
        });
        self.running_intent = Some(RunningIntent {
            label,
            intent,
            receiver,
        });
    }

    pub fn collect_finished_intent(&mut self) -> bool {
        let Some(running) = &self.running_intent else {
            return false;
        };
        let model = match running.receiver.try_recv() {
            Ok(model) => model,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.running_intent = None;
                return true;
            }
        };
        let running = self.running_intent.take().expect("running intent exists");
        let queued_during_run = self.model.take_pending_intents();
        self.model = model;
        self.model
            .append_pending_intents(remove_running_intent(queued_during_run, &running.intent));
        true
    }
}

fn remove_running_intent(
    mut intents: Vec<state::GuiActionIntent>,
    running: &state::GuiActionIntent,
) -> Vec<state::GuiActionIntent> {
    if let Some(index) = intents.iter().position(|intent| intent == running) {
        intents.remove(index);
    }
    intents
}

fn apply_dark_theme(ctx: &egui::Context, colors: UiColors) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = colors.canvas;
    visuals.window_fill = colors.surface_1;
    visuals.faint_bg_color = colors.surface_2;
    visuals.extreme_bg_color = colors.canvas;
    visuals.selection.bg_fill = colors.surface_3;
    visuals.selection.stroke = egui::Stroke::new(1.0, colors.focus);
    visuals.widgets.inactive.bg_fill = colors.surface_1;
    visuals.widgets.hovered.bg_fill = colors.surface_2;
    visuals.widgets.active.bg_fill = colors.surface_3;
    visuals.widgets.noninteractive.bg_fill = colors.surface_1;
    visuals.widgets.inactive.fg_stroke.color = colors.ink_muted;
    visuals.widgets.hovered.fg_stroke.color = colors.ink;
    visuals.widgets.active.fg_stroke.color = colors.ink;
    visuals.hyperlink_color = colors.ink_muted;
    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.interaction.selectable_labels = workbench_static_labels_selectable();
    ctx.set_style(style);
}

fn render_sidebar_nav_item(
    ui: &mut egui::Ui,
    view: NavigationView,
    selected: bool,
    colors: UiColors,
) -> egui::Response {
    render_sidebar_grid_item(
        ui,
        icons::navigation_icon(view),
        view.title(),
        selected,
        SIDEBAR_NAV_ROW_HEIGHT,
        colors,
    )
}

fn render_sidebar_scope_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    selected: bool,
    colors: UiColors,
) -> egui::Response {
    render_sidebar_grid_item(ui, icon, label, selected, SIDEBAR_SCOPE_ROW_HEIGHT, colors)
}

fn render_sidebar_boundary_extension(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let x = ui
        .painter()
        .round_to_pixel_center(rect.left() + SIDEBAR_WIDTH)
        - 1.0;
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;
    ui.painter().vline(x, rect.y_range(), stroke);
}

fn render_sidebar_section_label(ui: &mut egui::Ui, label: &str, colors: UiColors) {
    let grid = sidebar_grid_metrics();
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(grid.section_label_x);
        ui.label(
            egui::RichText::new(label)
                .color(colors.ink_subtle)
                .size(12.0),
        );
    });
    ui.add_space(4.0);
}

fn render_sidebar_grid_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    selected: bool,
    row_height: f32,
    colors: UiColors,
) -> egui::Response {
    let grid = sidebar_grid_metrics();
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, row_height), egui::Sense::click());
    let fill = workbench_row_fill(selected, response.hovered(), colors);
    let hover_alpha = if selected {
        1.0
    } else {
        ui.ctx().animate_bool_responsive(
            ui.id().with(("sidebar_row_hover", label)),
            response.hovered(),
        )
    };
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(grid.row_outer_inset, 2.0)),
            egui::Rounding::same(grid.row_radius),
            fade_color(fill, hover_alpha),
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(grid.row_outer_inset, 2.0)),
            egui::Rounding::same(grid.row_radius),
            egui::Stroke::new(1.0, colors.focus),
        );
    }

    let text_color = if selected {
        colors.ink
    } else {
        colors.ink_muted
    };
    ui.painter().text(
        egui::pos2(rect.left() + grid.icon_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(13.0),
        text_color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + grid.label_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    response
}

fn render_main(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    renderable: &RenderableView,
    colors: UiColors,
) {
    ui.add_space(10.0);
    render_main_heading(ui, &renderable.title, colors);
    ui.add_space(8.0);
    render_inset_gap(ui);
    ui.add_space(4.0);
    render_action_controls(ui, model, colors);
    if matches!(renderable.view, NavigationView::Skills) {
        render_skill_filters(ui, model, colors);
        ui.add_space(8.0);
    }

    if renderable.main_rows.is_empty() {
        ui.add_space(20.0);
        let grid = workbench_content_grid(ui.available_width());
        ui.horizontal(|ui| {
            ui.add_space(grid.left);
            ui.label(
                egui::RichText::new(renderable.empty_message.unwrap_or("No rows"))
                    .color(colors.ink_subtle),
            );
        });
    } else if matches!(renderable.view, NavigationView::Dashboard) {
        render_dashboard_overview(ui, renderable, colors);
    } else {
        render_workbench_table(ui, model, renderable, colors);
    }
}

fn render_main_heading(ui: &mut egui::Ui, title: &str, colors: UiColors) {
    let grid = dashboard_overview_grid(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(grid.heading_x);
        ui.heading(
            egui::RichText::new(title)
                .size(20.0)
                .color(colors.ink)
                .strong(),
        );
    });
}

fn render_inset_gap(ui: &mut egui::Ui) {
    ui.add_space(1.0);
}

fn render_dashboard_overview(ui: &mut egui::Ui, renderable: &RenderableView, colors: UiColors) {
    ui.add_space(4.0);
    let grid = dashboard_overview_grid(ui.available_width());

    ui.horizontal(|ui| {
        ui.add_space(grid.heading_x);
        ui.label(
            egui::RichText::new("Overview")
                .size(15.0)
                .strong()
                .color(colors.ink),
        );
    });
    ui.add_space(8.0);

    for row in &renderable.main_rows {
        render_dashboard_overview_row(ui, row, grid, colors);
    }
}

fn render_dashboard_overview_row(
    ui: &mut egui::Ui,
    row: &RenderRow,
    grid: DashboardOverviewGrid,
    colors: UiColors,
) {
    let row_width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row_width, MAIN_ROW_HEIGHT), egui::Sense::hover());
    let fill = workbench_row_fill(false, response.hovered(), colors);
    let hover_alpha = ui.ctx().animate_bool_responsive(
        ui.id().with(("dashboard_row_hover", row.id.as_str())),
        response.hovered(),
    );
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(grid.row_label_x, 0.0)),
            egui::Rounding::same(workbench_content_grid(row_width).row_rounding),
            fade_color(fill, hover_alpha),
        );
    }

    let mut row_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(grid.row_label_x, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let label = row.cells.first().map(String::as_str).unwrap_or("-");
    let value = row.cells.get(1).map(String::as_str).unwrap_or("-");
    row_ui.add_sized(
        [grid.label_width, MAIN_ROW_HEIGHT],
        egui::Label::new(egui::RichText::new(label).color(colors.ink_muted)).truncate(),
    );
    row_ui.add_sized(
        [grid.value_width, MAIN_ROW_HEIGHT],
        egui::Label::new(egui::RichText::new(value).color(colors.ink).strong()).truncate(),
    );
}

fn render_workbench_table(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    renderable: &RenderableView,
    colors: UiColors,
) {
    egui::ScrollArea::horizontal()
        .id_salt(format!("main_table_horizontal_{:?}", renderable.view))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let viewport_width = ui.available_width();
            let metrics = workbench_table_metrics(&renderable.columns, viewport_width);
            render_table_header(ui, &renderable.columns, &metrics, colors);
            render_inset_gap(ui);
            egui::ScrollArea::vertical()
                .id_salt(format!("main_table_vertical_{:?}", renderable.view))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for row in &renderable.main_rows {
                        render_table_row(ui, model, renderable, row, &metrics, colors);
                    }
                });
        });
}

fn render_table_header(
    ui: &mut egui::Ui,
    columns: &[String],
    metrics: &WorkbenchTableMetrics,
    colors: UiColors,
) {
    let header_height = 22.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(metrics.table_width, header_height),
        egui::Sense::hover(),
    );
    for ((column, left), width) in columns
        .iter()
        .zip(metrics.column_lefts.iter())
        .zip(metrics.column_widths.iter())
    {
        let cell_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + *left, rect.top()),
            egui::vec2(*width, header_height),
        );
        ui.put(
            cell_rect,
            egui::Label::new(
                egui::RichText::new(column)
                    .color(colors.ink_subtle)
                    .strong(),
            )
            .halign(cell_halign(column))
            .truncate(),
        );
    }
}

fn render_table_row(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    renderable: &RenderableView,
    row: &RenderRow,
    metrics: &WorkbenchTableMetrics,
    colors: UiColors,
) {
    let selected = is_render_row_selected(model, renderable.view, &row.id);
    let grid = workbench_content_grid(metrics.table_width);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(metrics.table_width, MAIN_ROW_HEIGHT),
        egui::Sense::click(),
    );
    let hovered = response.hovered() || response.has_focus();
    let fill = workbench_row_fill(selected, hovered, colors);
    let hover_alpha = if selected {
        1.0
    } else {
        ui.ctx().animate_bool_responsive(
            ui.id().with(("workbench_row_hover", row.id.as_str())),
            hovered,
        )
    };
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(grid.inset, 0.0)),
            egui::Rounding::same(grid.row_rounding),
            fade_color(fill, hover_alpha),
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(grid.inset, 0.0)).shrink(1.0),
            egui::Rounding::same(grid.row_rounding),
            egui::Stroke::new(1.0, colors.focus),
        );
    }

    for (((cell, column), left), width) in row
        .cells
        .iter()
        .zip(renderable.columns.iter())
        .zip(metrics.column_lefts.iter())
        .zip(metrics.column_widths.iter())
    {
        let cell_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + *left, rect.top()),
            egui::vec2(*width, MAIN_ROW_HEIGHT),
        )
        .translate(egui::vec2(workbench_cell_content_offset_x(column), 0.0));
        render_table_cell(ui, cell_rect, cell, column, selected, hovered, colors);
    }
    if let Some(disclosure) = plugin_row_disclosure(renderable.view, &renderable.columns) {
        ui.painter().text(
            egui::pos2(rect.right() - grid.inset - 12.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            disclosure,
            egui::FontId::proportional(12.0),
            if selected {
                colors.ink_muted
            } else {
                colors.ink_tertiary
            },
        );
    }

    let keyboard_activated = response.has_focus()
        && ui.input(|input| {
            [egui::Key::Enter, egui::Key::Space]
                .into_iter()
                .any(|key| workbench_row_accepts_keyboard_key(key) && input.key_pressed(key))
        });
    if response.clicked() || keyboard_activated {
        response.request_focus();
        model.select_render_row(&row.id);
    }
}

fn render_table_cell(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell: &str,
    column: &str,
    selected: bool,
    row_hovered: bool,
    colors: UiColors,
) {
    match workbench_cell_style(column) {
        WorkbenchCellStyle::StatusBadge => {
            let badge_rect = workbench_status_badge_rect(cell_rect, cell);
            let mut cell_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(badge_rect)
                    .layout(cell_layout(column)),
            );
            render_status_badge(&mut cell_ui, cell, row_hovered, selected, colors);
        }
        WorkbenchCellStyle::Mono => {
            ui.put(
                cell_rect,
                egui::Label::new(
                    egui::RichText::new(cell)
                        .monospace()
                        .size(12.0)
                        .color(colors.ink_subtle),
                )
                .halign(cell_halign(column))
                .truncate(),
            );
        }
        WorkbenchCellStyle::Text => {
            ui.put(
                cell_rect,
                egui::Label::new(egui::RichText::new(cell).color(colors.ink_muted))
                    .halign(cell_halign(column))
                    .truncate(),
            );
        }
    }
}

fn cell_halign(column: &str) -> egui::Align {
    match workbench_cell_alignment(column) {
        WorkbenchCellAlignment::Center => egui::Align::Center,
    }
}

fn cell_layout(column: &str) -> egui::Layout {
    match workbench_cell_alignment(column) {
        WorkbenchCellAlignment::Center => {
            egui::Layout::centered_and_justified(egui::Direction::TopDown)
        }
    }
}

fn render_status_badge(
    ui: &mut egui::Ui,
    value: &str,
    row_hovered: bool,
    selected: bool,
    colors: UiColors,
) -> egui::Response {
    let text_color = status_color(value, colors);
    egui::Frame::none()
        .fill(status_badge_fill(value, row_hovered, selected, colors))
        .stroke(status_badge_stroke(value, row_hovered, selected, colors))
        .rounding(egui::Rounding::same(3.0))
        .inner_margin(egui::Margin::symmetric(
            STATUS_BADGE_HORIZONTAL_PADDING,
            2.0,
        ))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(icons::status_icon(value)).color(text_color));
                ui.label(
                    egui::RichText::new(value)
                        .color(colors.ink_muted)
                        .size(12.0),
                );
            });
        })
        .response
}

fn fade_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    if color == egui::Color32::TRANSPARENT {
        return color;
    }
    let alpha = (255.0 * factor.clamp(0.0, 1.0)).round() as u8;
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn render_workbench_button(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    colors: UiColors,
    danger: bool,
    enabled: bool,
) -> egui::Response {
    let metrics = workbench_button_metrics();
    let text = workbench_button_label(icon, label);
    let width = (metrics.horizontal_padding * 2.0)
        + metrics.icon_width
        + ((label.chars().count() as f32) * 7.0)
        + 6.0;
    let text_color = if !enabled {
        colors.ink_tertiary
    } else if danger {
        colors.danger
    } else {
        colors.ink_muted
    };
    let stroke_color = if !enabled {
        colors.hairline
    } else if danger {
        colors.danger
    } else {
        colors.hairline
    };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(13.0).color(text_color))
            .min_size(egui::vec2(width.max(44.0), metrics.height))
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .rounding(egui::Rounding::same(metrics.radius)),
    );

    let hover_fill = workbench_button_hover_fill(enabled, response.hovered(), colors);
    if hover_fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            response.rect.shrink(1.0),
            egui::Rounding::same(metrics.radius),
            hover_fill,
        );
        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            workbench_button_label(icon, label),
            egui::FontId::proportional(13.0),
            text_color,
        );
        ui.painter().rect_stroke(
            response.rect.shrink(1.0),
            egui::Rounding::same(metrics.radius),
            egui::Stroke::new(1.0, stroke_color),
        );
    }
    if enabled && response.hovered() && !danger {
        ui.painter().rect_stroke(
            response.rect.shrink(1.0),
            egui::Rounding::same(metrics.radius),
            egui::Stroke::new(1.0, colors.hairline_strong),
        );
    }
    response
}

fn render_command_row(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash,
    icon: &str,
    label: &str,
    colors: UiColors,
    danger: bool,
    enabled: bool,
) -> egui::Response {
    let grid = workbench_content_grid(ui.available_width());
    let metrics = workbench_command_row_metrics();
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), metrics.height), sense);
    let row_rect = rect
        .shrink2(egui::vec2(grid.inset, 1.0))
        .translate(egui::vec2(0.0, 0.0));
    let hovered = enabled && (response.hovered() || response.has_focus());
    let hover_alpha = ui
        .ctx()
        .animate_bool_responsive(ui.id().with(("command_row_hover", id_source)), hovered);
    let fill = workbench_row_fill(false, hovered, colors);
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            row_rect,
            egui::Rounding::same(metrics.radius),
            fade_color(fill, hover_alpha),
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            row_rect.shrink(1.0),
            egui::Rounding::same(metrics.radius),
            egui::Stroke::new(1.0, colors.focus),
        );
    }

    let text_color = if !enabled {
        colors.ink_tertiary
    } else if danger {
        colors.danger
    } else {
        colors.ink_muted
    };
    ui.painter().text(
        egui::pos2(row_rect.left() + metrics.icon_x, row_rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(13.0),
        text_color,
    );
    ui.painter().text(
        egui::pos2(row_rect.left() + metrics.label_x, row_rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    response
}

fn render_confirmation_message(ui: &mut egui::Ui, message: &str, colors: UiColors) {
    let grid = workbench_content_grid(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(grid.left + workbench_command_row_metrics().label_x);
        ui.label(
            egui::RichText::new(message)
                .color(colors.warning)
                .size(12.0),
        );
    });
}

fn status_color(value: &str, colors: UiColors) -> egui::Color32 {
    match value {
        "Enabled" | "Ready" | "Yes" => colors.success,
        "Invalid" | "Missing" | "No" => colors.danger,
        "Disabled" | "Read-only" | "Not configured" => colors.info,
        value if value.contains("Invalid") || value.contains("Missing") => colors.danger,
        value if value.contains("Outdated") || value.contains("Drift") => colors.warning,
        _ => colors.info,
    }
}

fn table_column_widths(columns: &[String]) -> Vec<f32> {
    columns
        .iter()
        .map(|column| match column.as_str() {
            "Skill" | "Agent" => 156.0,
            "Project skill directories" | "Path" => 280.0,
            "Scope" | "Source" => 144.0,
            "Status" | "Toggle" | "Enabled" | "Writable" | "Managed" | "Validation" => 112.0,
            "Updated" | "Outdated" | "Drift" | "Risk" => 120.0,
            _ => 132.0,
        })
        .collect()
}

fn is_render_row_selected(model: &GuiModel, view: NavigationView, row_id: &str) -> bool {
    match view {
        NavigationView::Dashboard => false,
        NavigationView::Skills => model
            .selected_skill_instance()
            .is_some_and(|instance| instance.id == row_id),
        NavigationView::Agents => model
            .selected_agent()
            .is_some_and(|agent| agent.id.as_str() == row_id),
        NavigationView::Projects => {
            model
                .selected_skill_instance()
                .is_some_and(|instance| instance.id == row_id)
                || model
                    .selected_deployment()
                    .is_some_and(|deployment| deployment.id == row_id)
        }
        NavigationView::Plugins => {
            model
                .selected_plugin()
                .is_some_and(|plugin| plugin.id == row_id)
                || model
                    .selected_plugin_capability()
                    .is_some_and(|capability| capability.id == row_id)
        }
    }
}

fn render_skill_filters(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    let grid = workbench_content_grid(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(grid.left);
        ui.label(egui::RichText::new("Agent").color(colors.ink_subtle));
        let agent_text = model
            .skill_agent_filter()
            .and_then(|selected| {
                model
                    .skill_agent_filter_options()
                    .into_iter()
                    .find(|(agent_id, _)| agent_id == selected)
                    .map(|(_, label)| label)
            })
            .unwrap_or_else(|| "All".to_string());
        egui::ComboBox::from_id_salt("skill_agent_filter")
            .width(workbench_filter_width("Agent"))
            .selected_text(agent_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(model.skill_agent_filter().is_none(), "All")
                    .clicked()
                {
                    model.set_skill_agent_filter(None);
                }
                for (agent_id, label) in model.skill_agent_filter_options() {
                    if ui
                        .selectable_label(
                            model.skill_agent_filter() == Some(&agent_id),
                            label.as_str(),
                        )
                        .clicked()
                    {
                        model.set_skill_agent_filter(Some(agent_id));
                    }
                }
            });

        ui.label(egui::RichText::new("Scope").color(colors.ink_subtle));
        let scope_text = model.skill_scope_filter().unwrap_or("All").to_string();
        egui::ComboBox::from_id_salt("skill_scope_filter")
            .width(workbench_filter_width("Scope"))
            .selected_text(scope_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(model.skill_scope_filter().is_none(), "All")
                    .clicked()
                {
                    model.set_skill_scope_filter(None);
                }
                for scope in model.skill_scope_filter_options() {
                    if ui
                        .selectable_label(
                            model.skill_scope_filter() == Some(scope.as_str()),
                            &scope,
                        )
                        .clicked()
                    {
                        model.set_skill_scope_filter(Some(scope));
                    }
                }
            });

        ui.label(egui::RichText::new("Status").color(colors.ink_subtle));
        let status_text = model.skill_status_filter().unwrap_or("All").to_string();
        egui::ComboBox::from_id_salt("skill_status_filter")
            .width(workbench_filter_width("Status"))
            .selected_text(status_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(model.skill_status_filter().is_none(), "All")
                    .clicked()
                {
                    model.set_skill_status_filter(None);
                }
                for status in model.skill_status_filter_options() {
                    if ui
                        .selectable_label(model.skill_status_filter() == Some(status), status)
                        .clicked()
                    {
                        model.set_skill_status_filter(Some(status.to_string()));
                    }
                }
            });
    });
}

fn render_action_controls(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    if matches!(model.active_view, NavigationView::Dashboard) {
        return;
    }

    let grid = workbench_content_grid(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(grid.left);
        ui.label(
            egui::RichText::new("Actions")
                .color(colors.ink_subtle)
                .size(12.0),
        );
    });
    ui.add_space(4.0);
    match model.active_view {
        NavigationView::Skills => {
            render_skill_controls(ui, model, colors);
        }
        NavigationView::Projects => {
            render_project_controls(ui, model, colors);
        }
        NavigationView::Agents => {
            render_agent_editor_controls(ui, model, colors);
        }
        NavigationView::Plugins => {
            render_plugin_controls(ui, model, colors);
        }
        NavigationView::Dashboard => {}
    }
    ui.add_space(8.0);
}

fn render_project_controls(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    if let Some(draft) = model.open_project_draft().cloned() {
        let mut path_text = draft.path_text;
        render_path_field(
            ui,
            "Project path",
            &mut path_text,
            PathFieldKind::ExistingDirectory,
            colors,
        );
        model.update_open_project_path(path_text);
        if render_command_row(
            ui,
            "project_open",
            icons::BROWSE,
            "Open",
            colors,
            false,
            true,
        )
        .clicked()
        {
            let _ = model.request_save_open_project();
        }
        if render_command_row(
            ui,
            "project_open_cancel",
            icons::CANCEL,
            "Cancel",
            colors,
            false,
            true,
        )
        .clicked()
        {
            model.cancel_open_project();
        }
        return;
    }

    if render_command_row(
        ui,
        "project_open_project",
        icons::BROWSE,
        "Open project",
        colors,
        false,
        true,
    )
    .clicked()
    {
        model.begin_open_project();
    }
    if model.pending_remove_confirmation().is_some() {
        render_confirmation_message(ui, DRIFT_REMOVE_CONFIRMATION_MESSAGE, colors);
        if render_command_row(
            ui,
            "project_confirm_remove",
            icons::REMOVE,
            "Confirm Remove",
            colors,
            true,
            true,
        )
        .clicked()
        {
            let _ = model.confirm_pending_remove();
        }
    }
    for action in project_actions(model) {
        render_project_action_button(ui, model, colors, action);
    }
}

fn render_plugin_action_button(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    colors: UiColors,
    action: PluginAction,
) {
    let (icon, label) = match action {
        PluginAction::BackToPlugins => (icons::BACK, "Plugins"),
        PluginAction::ScanPlugins => (icons::SCAN, action.label()),
        PluginAction::Enable => (icons::ENABLE_PLUGIN, action.label()),
        PluginAction::Disable => (icons::DISABLE_PLUGIN, action.label()),
    };
    let danger = matches!(action, PluginAction::Disable);
    if !render_command_row(
        ui,
        ("plugin_action", label),
        icon,
        label,
        colors,
        danger,
        true,
    )
    .clicked()
    {
        return;
    }

    match action {
        PluginAction::BackToPlugins => {
            model.clear_plugin_selection();
        }
        PluginAction::ScanPlugins => {
            let _ = model.request_scan_plugins();
        }
        PluginAction::Enable => {
            let _ = model.request_enable_selected_plugin();
        }
        PluginAction::Disable => {
            let _ = model.request_disable_selected_plugin();
        }
    }
}

fn render_plugin_controls(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    for action in plugin_actions(model) {
        render_plugin_action_button(ui, model, colors, action);
    }
}

fn render_skill_action_button(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    colors: UiColors,
    action: SkillAction,
) {
    let clicked = render_command_row(
        ui,
        ("skill_action", action.label()),
        icons::skill_action_icon(action),
        skill_action_command_label(action, false),
        colors,
        matches!(action, SkillAction::Disable),
        true,
    )
    .clicked();

    if !clicked {
        return;
    }

    match action {
        SkillAction::ScanAgentSpaces => {
            let _ = model.request_scan_agent_spaces();
        }
        SkillAction::Enable => {
            let _ = model.request_enable_selected_skill_instance();
        }
        SkillAction::Disable => {
            let _ = model.request_disable_selected_skill_instance_with_confirmation(false);
        }
    }
}

fn render_skill_controls(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    if model
        .pending_disable_skill_instance_confirmation()
        .is_some()
    {
        render_confirmation_message(ui, SKILL_INSTANCE_DISABLE_CONFIRMATION_MESSAGE, colors);
        if render_command_row(
            ui,
            "skill_confirm_disable",
            icons::DISABLE_SKILL,
            skill_action_command_label(SkillAction::Disable, true),
            colors,
            true,
            true,
        )
        .clicked()
        {
            let _ = model.confirm_pending_disable_skill_instance();
        }
        ui.add_space(4.0);
    }

    for action in skill_actions(model) {
        render_skill_action_button(ui, model, colors, action);
    }
}

fn render_agent_action_button(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    colors: UiColors,
    action: AgentAction,
) {
    let clicked = render_command_row(
        ui,
        ("agent_action", action.label()),
        icons::agent_action_icon(action),
        action.label(),
        colors,
        matches!(action, AgentAction::RemoveCustom),
        true,
    )
    .clicked();
    if !clicked {
        return;
    }

    match action {
        AgentAction::EditSelected => {
            let _ = model.begin_edit_selected_agent_path();
        }
        AgentAction::ResetDefault => {
            let _ = model.request_reset_selected_agent_project_dirs();
        }
        AgentAction::RemoveCustom => {
            let _ = model.request_remove_selected_custom_agent();
        }
        AgentAction::AddCustom => {
            model.begin_add_custom_agent();
        }
    }
}

fn render_agent_editor_controls(ui: &mut egui::Ui, model: &mut GuiModel, colors: UiColors) {
    if let Some(draft) = model.agent_editor_draft().cloned() {
        let mut id_text = draft.id_text;
        let mut label_text = draft.label_text;
        let mut project_dir_text = draft.project_dir_text;
        let is_add = matches!(draft.mode, AgentEditorMode::AddCustom);

        if is_add {
            ui.label(egui::RichText::new("Agent id").color(colors.ink_subtle));
            ui.text_edit_singleline(&mut id_text);
            ui.label(egui::RichText::new("Label").color(colors.ink_subtle));
            ui.text_edit_singleline(&mut label_text);
        }
        render_path_field(
            ui,
            "Project Skill dir",
            &mut project_dir_text,
            PathFieldKind::AgentProjectDir,
            colors,
        );
        model.update_agent_editor_identity(id_text, label_text);
        model.update_agent_editor_project_dir(project_dir_text);

        if render_command_row(
            ui,
            "agent_editor_save",
            icons::SAVE,
            "Save",
            colors,
            false,
            true,
        )
        .clicked()
        {
            let _ = model.request_save_agent_editor();
        }
        if render_command_row(
            ui,
            "agent_editor_cancel",
            icons::CANCEL,
            "Cancel",
            colors,
            false,
            true,
        )
        .clicked()
        {
            model.cancel_agent_editor();
        }
        return;
    }

    for action in agent_actions(model) {
        render_agent_action_button(ui, model, colors, action);
    }
}

fn render_path_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    kind: PathFieldKind,
    colors: UiColors,
) {
    ui.label(egui::RichText::new(label).color(colors.ink_subtle));
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(value)
                .font(egui::TextStyle::Monospace)
                .desired_width(190.0),
        );
        if render_workbench_button(ui, icons::BROWSE, "Browse", colors, false, true).clicked() {
            if let Some(path) = browse_for_folder(value) {
                *value = path;
            }
        }
        let can_reveal = path_exists(value);
        if render_workbench_button(ui, icons::REVEAL, "Reveal", colors, false, can_reveal).clicked()
        {
            reveal_path(value);
        }
        if render_workbench_button(
            ui,
            icons::COPY,
            "Copy",
            colors,
            false,
            !value.trim().is_empty(),
        )
        .clicked()
        {
            ui.output_mut(|output| output.copied_text = value.trim().to_string());
        }
    });
    if let Some(message) = path_validation_message(value, kind) {
        ui.label(
            egui::RichText::new(message)
                .color(colors.warning)
                .size(12.0),
        );
    }
}

fn path_exists(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && Utf8Path::new(trimmed).exists()
}

#[cfg(target_os = "macos")]
fn browse_for_folder(current_value: &str) -> Option<String> {
    let mut script =
        "POSIX path of (choose folder with prompt \"Choose a Skill-kits folder\")".to_string();
    let trimmed = current_value.trim();
    if !trimmed.is_empty() && Utf8Path::new(trimmed).is_dir() {
        script = format!(
            "POSIX path of (choose folder with prompt \"Choose a Skill-kits folder\" default location POSIX file {})",
            applescript_string(trimmed)
        );
    }
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!path.is_empty()).then_some(path)
}

#[cfg(not(target_os = "macos"))]
fn browse_for_folder(_current_value: &str) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn reveal_path(value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    let _ = Command::new("open").arg("-R").arg(trimmed).status();
}

#[cfg(not(target_os = "macos"))]
fn reveal_path(_value: &str) {}

#[cfg(target_os = "macos")]
fn applescript_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn render_project_action_button(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    colors: UiColors,
    action: ProjectAction,
) {
    let clicked = render_command_row(
        ui,
        ("project_action", action.label()),
        icons::project_action_icon(action),
        action.label(),
        colors,
        matches!(action, ProjectAction::Remove),
        true,
    )
    .clicked();

    if !clicked {
        return;
    }

    match action {
        ProjectAction::Refresh => {
            let _ = model.request_refresh_selected_project();
        }
        ProjectAction::AdoptSelected => {
            let _ = model.request_adopt_selected_discovered_project_skill();
        }
        ProjectAction::AdoptAll => {
            let _ = model.request_adopt_all_discovered_for_selected_project();
        }
        ProjectAction::ImportAsNew => {
            let _ = model.request_import_selected_project_conflict_as_new();
        }
        ProjectAction::Skip => {
            let _ = model.skip_selected_project_conflict();
        }
        ProjectAction::Deploy => {
            let _ = model.request_deploy_selected_skill_to_target_agent();
        }
        ProjectAction::Enable => {
            let _ = model.request_enable_selected_deployment();
        }
        ProjectAction::Disable => {
            let _ = model.request_disable_selected_deployment();
        }
        ProjectAction::Redeploy => {
            let _ = model.request_redeploy_selected_deployment();
        }
        ProjectAction::Overwrite => {
            let _ = model.request_overwrite_selected_deployment();
        }
        ProjectAction::Promote => {
            let _ = model.request_promote_selected_deployment();
        }
        ProjectAction::Remove => {
            let _ = model.request_remove_selected_deployment(false);
        }
    }
}
