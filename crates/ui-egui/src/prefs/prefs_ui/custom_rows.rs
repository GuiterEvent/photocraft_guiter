//! The Preferences rows the app draws itself (`RuleKind::Custom`): memory, graphics, scratch disks, the lists with a
//! Clear button, the pressure curve, the tablet input and the Appearance page; and the Agents page
//! (`craft::automation::ui::agents`), drawn whole on the desktop. Every row edits the window's draft; nothing is
//! written before Apply or OK.

use craft::theme::variant::Tone;
use craft::theme::{ModeChoice, ThemeId};
use craft::theme_store::ui::AppearancePage;
use craft::theme_store::{SelectionChange, SuiteView};
use craft::ui::buttons::Button;
use craft::ui::choice::Checkbox;
use craft::ui::contract::{Bind, Entry, Props as _, Show as _, ShowContainer as _};
use craft::ui::display::Text;
use craft::ui::form::Field;
use craft::ui::patterns::preferences::PathSetting;
use craft::ui::pickers::Select;
use craft::ui::runtime::env::system_appearance;
use craft::ui::style::TypeRole;
use craft::ui::text::{NumberField, TextField};
use serde_json::{Value, json};

use super::fields::{PrefPages, choice_label};
use super::{PhotocraftApp, prefs};
use craft::ui::patterns::preferences::{CustomSettings, SettingsDraft};

const MEMORY_PERCENT: &str = "performance.memoryUsagePercent";
const MEMORY_MB: &str = "performance.memoryUsageMb";
const RENDERING_MODE: &str = "performance.renderingMode";
const USE_GPU: &str = "performance.useGpu";
const GPU_BACKEND: &str = "performance.gpuBackend";
const GPU_ADAPTER: &str = "performance.gpuAdapter";
const SCRATCH_DISKS: &str = "scratchDisks.disks";
const APPEARANCE_MODE: &str = "interface.appearanceMode";
const LIGHT_THEME: &str = "interface.lightTheme";
const DARK_THEME: &str = "interface.darkTheme";
const FAVORITE_THEMES: &str = "interface.favoriteThemes";
const AI_PROVIDER: &str = "integrations.aiProvider";
const GEMINI_API_KEY: &str = "integrations.geminiApiKey";
const GEMINI_MODEL: &str = "integrations.geminiModel";
#[cfg(not(target_arch = "wasm32"))]
const AGY_CLI_PATH: &str = "integrations.agyCliPath";
#[cfg(not(target_arch = "wasm32"))]
const GEMINI_CLI_PATH: &str = "integrations.geminiCliPath";
#[cfg(not(target_arch = "wasm32"))]
const CLAUDE_CLI_PATH: &str = "integrations.claudeCliPath";
#[cfg(not(target_arch = "wasm32"))]
const CODEX_CLI_PATH: &str = "integrations.codexCliPath";
#[cfg(not(target_arch = "wasm32"))]
const PHOTOCRAFT_CLI_PATH: &str = "integrations.photocraftCliPath";
/// The page agents' settings live on, drawn whole by [`craft::automation::ui::agents::AgentsPage`].
#[cfg(not(target_arch = "wasm32"))]
pub(super) const AGENTS_PAGE: &str = "integrations";
/// The scratch disk entry that names the system's temporary folder rather than a path.
const SYSTEM_TEMP: &str = "(system temp)";

/// The custom rows of the Preferences window. They read a copy of what they need from the app, taken before each pass
/// of the dialog host (`PrefsCustom::new`), because the dialog runs inside the host and has no app to read.
#[derive(Clone)]
pub(super) struct PrefsCustom {
    suite: SuiteView,
    picker_open: bool,
    lang: crate::i18n::Lang,
    gpu_info: Vec<String>,
    adapters: Vec<String>,
    installed_ram: Option<u64>,
    /// The agent link's state, for the Agents page.
    #[cfg(not(target_arch = "wasm32"))]
    agent_status: craft::automation::agents::AgentLinkStatus,
    /// The suite's `AI-MCP` flag: the Agents page points to the ArtCraft Launcher's switch.
    #[cfg(not(target_arch = "wasm32"))]
    mcp_flag: bool,
}

impl PrefsCustom {
    /// The rows for the next pass. `picker_open` is the dialog host's answer: whether a Color Picker opened from the
    /// window is open.
    pub(super) fn new(app: &PhotocraftApp, picker_open: bool) -> Self {
        let mut gpu_info = app.perf.gpu_info.lines();
        gpu_info.extend(gpu_memory_lines(app));
        PrefsCustom {
            suite: app.framework.suite().view(),
            picker_open,
            lang: crate::i18n::Lang::from_pref(&app.session.prefs().interface.language),
            gpu_info,
            adapters: app.perf.gpu_info.adapters.clone(),
            installed_ram: crate::gpu_canvas::physical_memory().filter(|n| *n > 0),
            #[cfg(not(target_arch = "wasm32"))]
            agent_status: app.services.agent_link.as_ref().map(|link| link.status().clone()).unwrap_or_default(),
            #[cfg(not(target_arch = "wasm32"))]
            mcp_flag: app.services.agent_switch.as_ref().is_some_and(|switch| switch.mcp_flag()),
        }
    }
}

impl CustomSettings<PrefPages> for PrefsCustom {
    fn row(&mut self, ui: &mut egui::Ui, key: &str, draft: &mut SettingsDraft) {
        match key {
            MEMORY_PERCENT => memory_percent_row(ui, draft),
            MEMORY_MB => self.memory_mb_row(ui, draft),
            RENDERING_MODE => rendering_mode_row(ui, draft),
            GPU_ADAPTER => self.gpu_rows(ui, draft),
            SCRATCH_DISKS => scratch_disks_row(ui, draft),
            "fileHandling.recentFiles" | "type.favoriteFonts" => self.clear_row(ui, key, draft),
            "tools.pressureCurve" => pressure_curve_row(ui, draft),
            "tools.tabletInput" => tablet_input_row(ui, draft),
            APPEARANCE_MODE => self.appearance_row(ui, draft),
            GEMINI_API_KEY => gemini_api_key_row(ui, draft),
            _ => {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn page(&mut self, ui: &mut egui::Ui, page: &str, draft: &mut SettingsDraft) {
        if page == AGENTS_PAGE {
            craft::automation::ui::agents::AgentsPage::new(AGENTS_PAGE, draft, &self.agent_status).mcp_flag(self.mcp_flag).show(ui);
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ai_assistant_settings_section(ui, draft);
        }
    }

    fn nested_open(&self) -> bool {
        self.picker_open
    }
}

/// The GPU memory budget lines of Performance › Graphics.
fn gpu_memory_lines(app: &PhotocraftApp) -> Vec<String> {
    let mut v = Vec::new();
    if app.perf.gpu_budget_mb > 0 {
        let budget = crate::chrome_ui::fmt_bytes(app.perf.gpu_budget_mb.saturating_mul(1 << 20));
        v.push(crate::i18n::fmt(tl!("GPU memory budget: {budget}"), &[("budget", &budget)]));
    }
    let doc = app.session.active().map(|st| st.doc.id.0);
    if let Some(fit) = app.perf.gpu_layer_fit.filter(|f| Some(f.doc) == doc) {
        let args = [("needed", crate::chrome_ui::fmt_bytes(fit.needed)), ("budget", crate::chrome_ui::fmt_bytes(fit.budget))];
        let args: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
        v.push(if fit.fits() {
            crate::i18n::fmt(tl!("This document's layers: {needed} of {budget} for layers"), &args)
        } else {
            crate::i18n::fmt(tl!("This document's layers need {needed} but may hold {budget}: whole-document redraws use the CPU"), &args)
        });
    }
    v
}

/// A line of help text under a row.
fn help(ui: &mut egui::Ui, text: &str) {
    Text::new(text.to_string()).role(TypeRole::Caption).show(ui);
}

/// A select over `(id, label)` options; the id picked this pass.
fn choose(ui: &mut egui::Ui, label: &str, salt: &'static str, current: String, options: &[(String, String)]) -> Option<String> {
    let entries: Vec<Entry<String>> = options.iter().map(|(id, name)| Entry::new(id.clone(), name.clone())).collect();
    let mut picked = None;
    Field::new(label.to_string()).id_salt(salt).show(ui, |ui, _| {
        picked = Select::new(label.to_string(), Bind::given(current.clone()), &entries).id_salt(salt).show(ui).value().cloned();
    });
    picked
}

/// The choices a preference path lists, with their shown labels.
fn choices_of(path: &str) -> Vec<(String, String)> {
    prefs::choices(path).unwrap_or(&[]).iter().map(|o| (o.to_string(), choice_label(o))).collect()
}

/// A checkbox enabling a 1–95 % field; off stores no value.
fn memory_percent_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    let stored: Option<u32> = draft.get(MEMORY_PERCENT);
    let mut on = stored.is_some();
    let mut percent = Some(f64::from(stored.unwrap_or(70)));
    let (was_on, was) = (on, percent);
    ui.horizontal(|ui| {
        Checkbox::new(Bind::bound(&mut on)).label(tl!("Memory Usage Percent")).show(ui);
        NumberField::new(tl!("Memory Usage Percent"), Bind::bound(&mut percent)).range(1.0..=95.0).unit("%").enabled(on).show(ui);
    });
    if (on, percent) == (was_on, was) {
        return;
    }
    if on {
        draft.set(MEMORY_PERCENT, &(percent.unwrap_or(70.0).clamp(1.0, 95.0).round() as u32));
    } else {
        // A TOML table holds no null: off removes the key.
        draft.remove(MEMORY_PERCENT);
    }
}

impl PrefsCustom {
    /// The per-document RAM limit with the installed-RAM line and its help.
    fn memory_mb_row(&self, ui: &mut egui::Ui, draft: &mut SettingsDraft) {
        let stored: u64 = draft.get(MEMORY_MB).unwrap_or(8192);
        let mut mb = Some(stored as f64);
        Text::new(tl!("Memory Usage").to_string()).role(TypeRole::BodyStrong).show(ui);
        NumberField::new(tl!("Per-document RAM limit (MB)"), Bind::bound(&mut mb)).range(256.0..=1_048_576.0).unit(" MB").show(ui);
        let mb = mb.unwrap_or(stored as f64).clamp(256.0, 1_048_576.0).round() as u64;
        if mb != stored {
            draft.set(MEMORY_MB, &mb);
        }
        if let Some(ram) = self.installed_ram {
            let installed = ram / (1 << 20);
            let percent = mb as f64 * 100.0 / installed as f64;
            help(
                ui,
                &crate::i18n::fmt(
                    tl!("Installed RAM: {ram} MB; this limit: {percent}%"),
                    &[("ram", &installed.to_string()), ("percent", &format!("{percent:.1}"))],
                ),
            );
        }
        help(ui, tl!("Limits each open document's pixels plus History. Oldest undo states are discarded when the limit is exceeded."));
        help(ui, tl!("Applies immediately when saved. Memory is not reserved in advance."));
        help(
            ui,
            tl!(
                "With GPU Memory set to Automatic, this also limits the GPU layer budget. That budget is capped at one quarter of installed RAM and, when known, three eighths of the graphics card's memory, with a 512 MB minimum. Raising this limit beyond those caps does not increase the GPU budget."
            ),
        );
    }

    /// The Graphics status block: GPU info lines, then the backend and adapter under Advanced.
    fn gpu_rows(&self, ui: &mut egui::Ui, draft: &mut SettingsDraft) {
        Text::new(tl!("Graphics").to_string()).role(TypeRole::BodyStrong).show(ui);
        for line in &self.gpu_info {
            Text::new(line.clone()).show(ui);
        }
        help(ui, tl!("Applies at next launch."));
        egui::CollapsingHeader::new(tl!("Advanced")).show(ui, |ui| {
            let backend: String = draft.get(GPU_BACKEND).unwrap_or_else(|| "auto".into());
            if let Some(next) = choose(ui, tl!("GPU Backend"), "graphics-backend", backend, &choices_of(GPU_BACKEND)) {
                draft.set(GPU_BACKEND, &next);
            }
            if prefs::is_hidden(GPU_ADAPTER) {
                return;
            }
            self.adapter_row(ui, draft);
        });
    }

    /// Which GPU draws on a machine with several. A saved adapter no listed one matches stays listed, marked ⚠, so
    /// opening the window doesn't change the choice. A saved name fragment ("nvidia") that matches one is not marked.
    fn adapter_row(&self, ui: &mut egui::Ui, draft: &mut SettingsDraft) {
        let current: String = draft.get(GPU_ADAPTER).unwrap_or_default();
        let mut names = self.adapters.clone();
        let kept = !current.is_empty() && !names.contains(&current);
        let missing = kept && !names.iter().any(|n| prefs::gpu_adapter_matches(n, &current));
        if kept {
            names.push(current.clone());
        }
        let mut options = vec![(String::new(), tl!("Automatic").to_string())];
        options.extend(names.into_iter().map(|n| {
            let shown = if missing && n == current { format!("⚠ {n}") } else { n.clone() };
            (n, shown)
        }));
        if let Some(next) = choose(ui, tl!("GPU adapter"), "graphics-adapter", current, &options) {
            draft.set(GPU_ADAPTER, &next);
        }
        help(ui, tl!("Applies at next launch."));
    }

    /// The count of a list and a Clear button.
    fn clear_row(&self, ui: &mut egui::Ui, key: &str, draft: &mut SettingsDraft) {
        let n = draft.get::<Vec<Value>>(key).map_or(0, |items| items.len());
        help(ui, &crate::i18n::trn(self.lang, n as u64, "{n} item", "{n} items"));
        if n > 0 && Button::new(tl!("Clear")).show(ui).clicked() {
            draft.set(key, &Vec::<String>::new());
        }
    }

    /// The Appearance page over the suite. Its selection goes into the draft's own keys, so OK and Apply commit it like
    /// any row and Cancel drops it; `Framework::sync_theme` sends the saved keys to the suite.
    fn appearance_row(&self, ui: &mut egui::Ui, draft: &mut SettingsDraft) {
        let theme = |key: &str| draft.get::<String>(key).and_then(|s| ThemeId::parse(&s).ok());
        let mode = match draft.get::<String>(APPEARANCE_MODE).as_deref() {
            Some("light") => ModeChoice::Light,
            Some("dark") => ModeChoice::Dark,
            _ => ModeChoice::System,
        };
        let shown = SelectionChange { mode: Some(mode), theme_light: theme(LIGHT_THEME), theme_dark: theme(DARK_THEME), ..Default::default() };
        let mut change = shown.clone();
        let favorites: Vec<ThemeId> = draft.get::<Vec<String>>(FAVORITE_THEMES).unwrap_or_default().iter().filter_map(|s| ThemeId::parse(s).ok()).collect();
        // Unknown OS answer previews light, which is what Auto resolves to then.
        let system_dark = system_appearance(ui.ctx()).appearance.mode == Some(craft::theme::Mode::Dark);
        let out = AppearancePage::new(&self.suite, Bind::bound(&mut change)).favorites(&favorites).text_size(false).system_dark(system_dark).show(ui);
        if change != shown {
            if change.mode != shown.mode {
                let name = match change.mode {
                    Some(ModeChoice::Light) => "light",
                    Some(ModeChoice::Dark) => "dark",
                    _ => "auto",
                };
                draft.set(APPEARANCE_MODE, &name);
            }
            if change.theme_light != shown.theme_light
                && let Some(id) = &change.theme_light
            {
                draft.set(LIGHT_THEME, &id.as_str());
            }
            if change.theme_dark != shown.theme_dark
                && let Some(id) = &change.theme_dark
            {
                draft.set(DARK_THEME, &id.as_str());
            }
        }
        if let Some(id) = out.favorite_toggled {
            let mut saved: Vec<String> = draft.get(FAVORITE_THEMES).unwrap_or_default();
            if let Some(at) = saved.iter().position(|f| f == id.as_str()) {
                saved.remove(at);
            } else {
                saved.push(id.as_str().to_owned());
            }
            draft.set(FAVORITE_THEMES, &saved);
        }
    }
}

/// One user-facing choice; legacy flags remain compatible with older settings.
pub(super) fn rendering_mode_value(draft: &SettingsDraft) -> String {
    draft.get::<String>(RENDERING_MODE).unwrap_or_else(|| {
        if draft.get::<bool>(USE_GPU) == Some(false) || draft.get::<String>(GPU_BACKEND).as_deref() == Some("cpu") { "cpu".into() } else { "auto".into() }
    })
}

/// Automatic / GPU / CPU; a change also writes `useGpu`, which older builds read.
fn rendering_mode_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    let options = [
        ("auto".to_string(), tl!("Automatic (recommended)").to_string()),
        ("gpu".to_string(), tl!("GPU").to_string()),
        ("cpu".to_string(), tl!("CPU / Compatibility").to_string()),
    ];
    let current = rendering_mode_value(draft);
    if let Some(next) = choose(ui, tl!("Rendering Mode"), "rendering-mode", current.clone(), &options)
        && next != current
    {
        draft.set(USE_GPU, &(next != "cpu"));
        draft.set(RENDERING_MODE, &next);
    }
    help(ui, tl!("Automatic uses GPU acceleration when available and falls back to CPU rendering on errors."));
    help(ui, tl!("Applies at next launch."));
}

/// Per disk a checkbox and a path, an Add disk button, and the warning when the scratch directory reports no free space.
fn scratch_disks_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    let disks: Vec<Value> = draft.get(SCRATCH_DISKS).unwrap_or_default();
    for (i, disk) in disks.iter().enumerate() {
        let on = disk.get("enabled").and_then(Value::as_bool).unwrap_or(false);
        let path = disk.get("path").and_then(Value::as_str).unwrap_or("").to_string();
        let path_key = format!("{SCRATCH_DISKS}[{i}].path");
        ui.horizontal(|ui| {
            let mut next = on;
            Checkbox::new(Bind::bound(&mut next)).label(tl!("Enabled")).id_salt(("disk-enabled", i)).show(ui);
            if next != on {
                draft.set(&format!("{SCRATCH_DISKS}[{i}].enabled"), &next);
            }
            // The system's temporary folder is named, not browsed for.
            if path.trim() == SYSTEM_TEMP {
                let mut typed = path.clone();
                TextField::new(tl!("Path"), Bind::bound(&mut typed)).id_salt(("disk-path", i)).show(ui);
                if typed != path {
                    draft.set(&path_key, &typed);
                }
            } else {
                PathSetting::new(tl!("Path"), &path_key, draft, craft::host::PickRequest::folder("scratch")).show(ui);
            }
        });
    }
    if Button::new(tl!("Add disk")).show(ui).clicked() {
        let mut disks: Vec<Value> = draft.get(SCRATCH_DISKS).unwrap_or_default();
        disks.push(json!({"enabled": true, "path": ""}));
        draft.set(SCRATCH_DISKS, &disks);
    }
    let scratch = photocraft_raster::spill::stats();
    if scratch.free_space_unmeasured {
        let dir = scratch.dir.as_deref().map(|p| p.display().to_string()).unwrap_or_else(|| tl!("Unknown").into());
        let warning = crate::i18n::fmt(
            tl!(
                "The current scratch directory ({path}) doesn't report its free space, so PhotoCraft can't stop before it fills up, and large files get no extra room from it without a Scratch Size Limit. This warning applies to the current configuration; changes above take effect on Apply or OK. Set a limit or use a local disk."
            ),
            &[("path", &dir)],
        );
        Text::new(warning).tone(Tone::Warn).show(ui);
    }
}

/// The pen's pressure response.
fn pressure_curve_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    let mut points: Vec<[f32; 2]> = draft.get("tools.pressureCurve").unwrap_or_default();
    let events = crate::pressure_curve_ui::editor(ui, tl!("Pressure Curve"), &mut points);
    if let Some(next) = events.value() {
        draft.set("tools.pressureCurve", next);
    }
}

/// The tablet input choice, with the WinTab note; Windows only.
fn tablet_input_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    if !cfg!(target_os = "windows") {
        return;
    }
    let current: String = draft.get("tools.tabletInput").unwrap_or_default();
    if let Some(next) = choose(ui, tl!("Tablet Input"), "tablet-input", current, &choices_of("tools.tabletInput")) {
        draft.set("tools.tabletInput", &next);
    }
    help(
        ui,
        tl!(
            "WinTab reads pressure, tilt, rotation and the eraser end from the tablet driver. Turn off Windows Ink in your tablet's software utility (for example Wacom Center) before using it."
        ),
    );
    help(ui, tl!("Applies at next launch."));
}

/// Password-masked Gemini API key row.
fn gemini_api_key_row(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    let current: String = draft.get(GEMINI_API_KEY).unwrap_or_default();
    let mut edited = current.clone();
    Field::new(tl!("Gemini API Key").to_string()).id_salt("gemini-api-key").show(ui, |ui, _| {
        TextField::new(tl!("Gemini API Key"), Bind::bound(&mut edited))
            .masked()
            .id_salt("gemini-api-key-input")
            .show(ui);
    });
    if edited != current {
        draft.set(GEMINI_API_KEY, &edited);
    }
}

/// Persistent UI-temp state for the non-blocking CLI environment check inside Preferences.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct CliEnvCheckState {
    running: bool,
    rx: Option<std::sync::mpsc::Receiver<crate::cli_agent::CliEnvReport>>,
    report: Option<crate::cli_agent::CliEnvReport>,
    error: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn cli_env_state_id() -> egui::Id {
    egui::Id::unique("photocraft-prefs-cli-env-check")
}

#[cfg(not(target_arch = "wasm32"))]
type SharedCliEnvState = std::sync::Arc<std::sync::Mutex<CliEnvCheckState>>;

#[cfg(not(target_arch = "wasm32"))]
fn shared_cli_env_state(ctx: &egui::Context) -> SharedCliEnvState {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(cli_env_state_id(), || {
            std::sync::Arc::new(std::sync::Mutex::new(CliEnvCheckState::default()))
        })
        .clone()
    })
}

/// Optional custom executable path field (leave blank for automatic PATH / install-dir detection).
#[cfg(not(target_arch = "wasm32"))]
fn cli_path_override_row(
    ui: &mut egui::Ui,
    draft: &mut SettingsDraft,
    key: &str,
    label: &str,
    salt: &'static str,
) {
    let current: String = draft.get(key).unwrap_or_default();
    let mut edited = current.clone();
    Field::new(label.to_string()).id_salt(salt).show(ui, |ui, _| {
        TextField::new(label.to_string(), Bind::bound(&mut edited))
            .placeholder(tl!("Auto-detect (leave blank)"))
            .limit(None)
            .id_salt((salt, "input"))
            .show(ui);
    });
    if edited != current {
        draft.set(key, &edited);
    }
}

/// Polls any in-flight CLI environment check and renders the check button and results list.
#[cfg(not(target_arch = "wasm32"))]
fn cli_environment_check_block(ui: &mut egui::Ui, draft: &SettingsDraft) {
    let ctx = ui.ctx().clone();
    let shared = shared_cli_env_state(&ctx);

    let (running, report, error) = {
        let mut guard = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(rx) = guard.rx.as_ref() {
            match rx.try_recv() {
                Ok(done) => {
                    guard.running = false;
                    guard.rx = None;
                    guard.report = Some(done);
                    guard.error = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    guard.running = false;
                    guard.rx = None;
                }
            }
        }
        (guard.running, guard.report.clone(), guard.error.clone())
    };

    ui.add_space(6.0);
    let button_label = if running { tl!("Checking CLI Environment…") } else { tl!("Check CLI Environment") };
    if Button::new(button_label).enabled(!running).id_salt("check-cli-env-btn").show(ui).clicked() {
        let cfg = crate::cli_agent::CliEnvCheckConfig {
            agy_path: draft.get(AGY_CLI_PATH).unwrap_or_default(),
            gemini_path: draft.get(GEMINI_CLI_PATH).unwrap_or_default(),
            claude_path: draft.get(CLAUDE_CLI_PATH).unwrap_or_default(),
            codex_path: draft.get(CODEX_CLI_PATH).unwrap_or_default(),
            photocraft_cli_path: draft.get(PHOTOCRAFT_CLI_PATH).unwrap_or_default(),
        };
        let wake_ctx = ctx.clone();
        let wake: crate::ai_backend::WakeCallback = std::sync::Arc::new(move || wake_ctx.request_repaint());
        let mut guard = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        match crate::cli_agent::start_cli_environment_check(cfg, wake) {
            Ok(rx) => {
                guard.running = true;
                guard.rx = Some(rx);
                guard.error = None;
            }
            Err(err) => {
                guard.running = false;
                guard.rx = None;
                guard.error = Some(err);
            }
        }
    }

    if let Some(err) = error {
        Text::new(err).tone(Tone::Bad).show(ui);
    }

    if let Some(report) = report {
        ui.add_space(4.0);
        for entry in &report.entries {
            render_cli_env_entry(ui, entry);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_cli_env_entry(ui: &mut egui::Ui, entry: &crate::cli_agent::CliEnvEntry) {
    let (status_line, tone) = match &entry.status {
        crate::cli_agent::CliDetectionStatus::Found(path) => {
            let suffix = match (&entry.version, &entry.error) {
                (Some(ver), _) => format!(" ({ver})"),
                (None, Some(err)) => format!(" ({err})"),
                (None, None) => String::new(),
            };
            let source = if entry.custom_override_used { tl!("custom") } else { tl!("auto") };
            let line = format!(
                "{} ({}): {} [{source}]{suffix} — {}",
                entry.label,
                entry.executable,
                tl!("Found"),
                path.display()
            );
            let tone = if entry.error.is_some() { Tone::Warn } else { Tone::Good };
            (line, tone)
        }
        crate::cli_agent::CliDetectionStatus::NotFound => {
            let hint = if entry.custom_override_used {
                tl!("Custom path not found")
            } else {
                tl!("Not found")
            };
            (format!("{} ({}): {hint}", entry.label, entry.executable), Tone::Warn)
        }
        crate::cli_agent::CliDetectionStatus::UnsupportedPlatform => (
            format!("{} ({}): {}", entry.label, entry.executable, tl!("Desktop only")),
            Tone::Neutral,
        ),
    };
    Text::new(status_line).role(TypeRole::Caption).tone(tone).show(ui);
}

/// AI Assistant settings shown below the Agents page on desktop.
#[cfg(not(target_arch = "wasm32"))]
fn ai_assistant_settings_section(ui: &mut egui::Ui, draft: &mut SettingsDraft) {
    Text::new(tl!("AI Assistant").to_string()).role(TypeRole::BodyStrong).show(ui);
    let current_provider: String = draft.get(AI_PROVIDER).unwrap_or_else(|| prefs::AiProvider::default().name().to_string());
    let provider_options: Vec<(String, String)> = choices_of(AI_PROVIDER)
        .into_iter()
        .map(|(id, label)| (id, tl!(&label).to_string()))
        .collect();
    if let Some(next) = choose(ui, tl!("Default AI Provider"), "ai-provider", current_provider.clone(), &provider_options)
        && next != current_provider
    {
        draft.set(AI_PROVIDER, &next);
    }
    help(
        ui,
        tl!("Local CLI providers (Antigravity, Gemini, Claude, Codex) sign in with your own account and require no API credits."),
    );

    ui.add_space(6.0);
    Text::new(tl!("CLI Executable Paths (Optional)").to_string()).role(TypeRole::BodyStrong).show(ui);
    help(
        ui,
        tl!("Leave blank to auto-detect from PATH and standard install folders. You may enter either an executable file or its folder."),
    );
    cli_path_override_row(ui, draft, AGY_CLI_PATH, tl!("Antigravity CLI (agy)"), "agy-cli-path");
    cli_path_override_row(ui, draft, GEMINI_CLI_PATH, tl!("Gemini CLI (gemini)"), "gemini-cli-path");
    cli_path_override_row(ui, draft, CLAUDE_CLI_PATH, tl!("Claude CLI (claude)"), "claude-cli-path");
    cli_path_override_row(ui, draft, CODEX_CLI_PATH, tl!("Codex CLI (codex)"), "codex-cli-path");
    cli_path_override_row(ui, draft, PHOTOCRAFT_CLI_PATH, tl!("PhotoCraft CLI (photocraft-cli)"), "photocraft-cli-path");

    cli_environment_check_block(ui, draft);

    ui.add_space(8.0);
    Text::new(tl!("Gemini API").to_string()).role(TypeRole::BodyStrong).show(ui);
    gemini_api_key_row(ui, draft);
    let current_model: String = draft.get(GEMINI_MODEL).unwrap_or_else(prefs::default_gemini_model);
    let options: Vec<(String, String)> = prefs::GEMINI_MODELS.iter().map(|m| ((*m).to_string(), (*m).to_string())).collect();
    if let Some(next) = choose(ui, tl!("Gemini Model"), "gemini-model", current_model.clone(), &options)
        && next != current_model
    {
        draft.set(GEMINI_MODEL, &next);
    }
}
