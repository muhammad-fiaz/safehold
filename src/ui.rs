#[cfg(feature = "gui")]
use anyhow::{Context, Result, anyhow};
#[cfg(feature = "gui")]
use eframe::{App, egui};
#[cfg(feature = "gui")]
use egui::{Align, Align2, Color32, Layout, RichText, Ui, Vec2};
#[cfg(feature = "gui")]
use std::collections::{BTreeMap, HashMap};
#[cfg(feature = "gui")]
use std::fs;
#[cfg(feature = "gui")]
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::time::{Duration, Instant};

#[cfg(feature = "gui")]
use crate::cli::CreateArgs;
use crate::cli::{ExportArgs, RunArgs};
#[cfg(feature = "gui")]
use crate::config::{self, Config, env_enc_path, lock_path};
#[cfg(feature = "gui")]
use crate::crypto::{self, LockInfo};
use crate::envops;
#[cfg(feature = "gui")]
use crate::store::{self};

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
enum TabView {
    Projects,
    Credentials,
    Global,
    Export,
    Settings,
    Statistics,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
struct NotificationMessage {
    text: String,
    severity: NotificationSeverity,
    timestamp: Instant,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
enum NotificationSeverity {
    Success,
    Warning,
    Error,
    Info,
}

#[cfg(feature = "gui")]
struct SafeHoldApp {
    cfg: Config,
    selected: Option<String>,
    maps_cache: HashMap<String, BTreeMap<String, String>>,
    passwords: HashMap<String, String>,

    // UI State
    current_tab: TabView,
    search_filter: String,
    show_passwords: bool,
    auto_save_interval: u64,

    // Create project dialog
    show_create: bool,
    new_project_name: String,
    new_project_lock: bool,
    new_project_password: String,
    new_project_confirm_password: String,

    // Add credential dialog
    show_add_credential: bool,
    new_key: String,
    new_val: String,

    // Export dialog
    show_export: bool,
    export_project: String,
    export_format: String,
    export_file: String,

    // Run command dialog
    show_run_command: bool,
    run_project: String,
    run_command: String,

    // Update credential dialog
    show_update_credential: bool,
    update_project: String,
    update_key: String,
    update_value: String,

    // Delete confirmation dialog
    show_delete_confirm: bool,
    delete_type: String, // "project" or "credential" or "global"
    delete_target: String,
    delete_project: String, // for credential deletion

    // Global credentials
    global_credentials: BTreeMap<String, String>,

    // Global operations dialogs
    show_add_global: bool,
    show_update_global: bool,
    global_new_key: String,
    global_new_value: String,
    global_update_key: String,
    global_update_value: String,

    // Destructive operations dialogs
    show_clean_cache_confirm: bool,
    show_delete_all_confirm: bool,
    show_about_dialog: bool,

    // Notifications
    notifications: Vec<NotificationMessage>,

    // Statistics
    total_projects: usize,
    total_credentials: usize,
    duplicate_keys: Vec<(String, Vec<String>)>,

    // Master Lock functionality
    master_lock_enabled: bool,
    show_master_lock_dialog: bool,
    master_lock_action: Option<bool>, // Some(true) = enable, Some(false) = disable, None = status
    master_password_input: String,
    master_password_confirm: String,

    // App Settings
    #[allow(dead_code)]
    app_settings: crate::app_settings::AppSettings,
}

#[cfg(feature = "gui")]
impl SafeHoldApp {
    fn new() -> Self {
        let cfg = config::load_config().unwrap_or_default();
        Self {
            cfg,
            selected: None,
            maps_cache: HashMap::new(),
            passwords: HashMap::new(),
            current_tab: TabView::Projects,
            search_filter: String::new(),
            show_passwords: false,
            auto_save_interval: 30,
            show_create: false,
            new_project_name: String::new(),
            new_project_lock: false,
            new_project_password: String::new(),
            new_project_confirm_password: String::new(),
            show_add_credential: false,
            new_key: String::new(),
            new_val: String::new(),
            show_export: false,
            export_project: String::new(),
            export_format: "env".to_string(),
            export_file: String::new(),
            show_run_command: false,
            run_project: String::new(),
            run_command: String::new(),
            show_update_credential: false,
            update_project: String::new(),
            update_key: String::new(),
            update_value: String::new(),
            show_delete_confirm: false,
            delete_type: String::new(),
            delete_target: String::new(),
            delete_project: String::new(),
            global_credentials: BTreeMap::new(),
            show_add_global: false,
            show_update_global: false,
            global_new_key: String::new(),
            global_new_value: String::new(),
            global_update_key: String::new(),
            global_update_value: String::new(),
            show_clean_cache_confirm: false,
            show_delete_all_confirm: false,
            show_about_dialog: false,
            notifications: Vec::new(),
            total_projects: 0,
            total_credentials: 0,
            duplicate_keys: Vec::new(),
            master_lock_enabled: crate::master_lock::is_master_lock_enabled(),
            show_master_lock_dialog: false,
            master_lock_action: None,
            master_password_input: String::new(),
            master_password_confirm: String::new(),
            app_settings: crate::app_settings::load_settings().unwrap_or_default(),
        }
    }

    fn add_notification(&mut self, text: String, severity: NotificationSeverity) {
        self.notifications.push(NotificationMessage {
            text,
            severity,
            timestamp: Instant::now(),
        });
        // Keep only last 5 notifications
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }

    fn select(&mut self, id_or_global: &str) {
        self.selected = Some(id_or_global.to_string());
        let _ = self.ensure_loaded(id_or_global);
        self.current_tab = TabView::Credentials;
    }

    fn ensure_loaded(&mut self, id_or_global: &str) -> Result<()> {
        if self.maps_cache.contains_key(id_or_global) {
            return Ok(());
        }
        let dir = if id_or_global == "global" {
            config::global_dir()?
        } else {
            config::set_dir(id_or_global)?
        };
        let pwd = self.passwords.get(id_or_global).map(|s| s.as_str());
        match read_env_map_dir(&dir, pwd) {
            Ok(map) => {
                self.maps_cache.insert(id_or_global.to_string(), map);
                Ok(())
            }
            Err(e) => {
                self.add_notification(
                    format!("Failed to load {}: {}", id_or_global, e),
                    NotificationSeverity::Error,
                );
                Err(e)
            }
        }
    }

    fn is_locked(&self, id_or_global: &str) -> bool {
        let dir = match if id_or_global == "global" {
            config::global_dir()
        } else {
            config::set_dir(id_or_global)
        } {
            Ok(d) => d,
            Err(_) => return false,
        };
        lock_path(&dir).exists()
    }

    fn add_kv(&mut self, id_or_global: &str) {
        if self.new_key.trim().is_empty() {
            self.add_notification(
                "Key cannot be empty".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        let map = self.maps_cache.entry(id_or_global.to_string()).or_default();
        if map.contains_key(&self.new_key) {
            self.add_notification(
                format!("Key '{}' already exists. Use edit to modify.", self.new_key),
                NotificationSeverity::Warning,
            );
            return;
        }

        map.insert(self.new_key.trim().to_string(), self.new_val.clone());
        if let Err(e) = self.save_map(id_or_global) {
            self.add_notification(format!("Save failed: {}", e), NotificationSeverity::Error);
            return;
        }

        self.add_notification(
            format!("Added credential '{}'", self.new_key),
            NotificationSeverity::Success,
        );
        self.new_key.clear();
        self.new_val.clear();
    }

    fn save_map(&mut self, id_or_global: &str) -> Result<()> {
        let dir = if id_or_global == "global" {
            config::global_dir()?
        } else {
            config::set_dir(id_or_global)?
        };
        let pwd = self.passwords.get(id_or_global).map(|s| s.as_str());
        if let Some(map) = self.maps_cache.get(id_or_global) {
            write_env_map_dir(&dir, map, pwd)
        } else {
            Ok(())
        }
    }

    fn delete_key(&mut self, id_or_global: &str, key: &str) {
        if let Some(map) = self.maps_cache.get_mut(id_or_global) {
            map.remove(key);
            if let Err(e) = self.save_map(id_or_global) {
                self.add_notification(format!("Delete failed: {}", e), NotificationSeverity::Error);
            } else {
                self.add_notification(
                    format!("Deleted credential '{}'", key),
                    NotificationSeverity::Success,
                );
                self.update_statistics();
            }
        }
    }

    fn refresh_config(&mut self) {
        if let Ok(cfg) = config::load_config() {
            self.cfg = cfg;
            self.update_statistics();
        }
    }

    fn update_statistics(&mut self) {
        self.total_projects = self.cfg.sets.len();
        self.total_credentials = self.maps_cache.values().map(|m| m.len()).sum();

        // Find duplicate keys across projects
        let mut all_keys: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        // Global project
        if let Some(map) = self.maps_cache.get("global") {
            for key in map.keys() {
                all_keys
                    .entry(key.clone())
                    .or_default()
                    .push("global".to_string());
            }
        }

        // User projects
        for project in &self.cfg.sets {
            if let Some(map) = self.maps_cache.get(&project.id) {
                for key in map.keys() {
                    all_keys
                        .entry(key.clone())
                        .or_default()
                        .push(project.id.clone());
                }
            }
        }

        self.duplicate_keys = all_keys
            .into_iter()
            .filter(|(_, projects)| projects.len() > 1)
            .collect();
    }

    fn setup_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Enhance spacing and sizing
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        // style.spacing.menu_margin = egui::Margin::same(8.0); // Commented out due to API changes

        // Enhanced colors
        style.visuals.override_text_color = Some(Color32::from_gray(240));
        style.visuals.hyperlink_color = Color32::from_rgb(100, 150, 250);
        style.visuals.selection.bg_fill = Color32::from_rgb(0, 120, 180);

        // Button styling
        style.visuals.widgets.inactive.bg_fill = Color32::from_gray(60);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 100, 160);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(0, 120, 180);

        ctx.set_style(style);
    }

    fn lock_unlock_global(&mut self, lock: bool) {
        match (lock, self.is_locked("global")) {
            (true, false) => {
                if self.new_project_password.is_empty() {
                    self.add_notification(
                        "Enter password before locking".to_string(),
                        NotificationSeverity::Warning,
                    );
                    return;
                }
                if self.new_project_password != self.new_project_confirm_password {
                    self.add_notification(
                        "Passwords do not match".to_string(),
                        NotificationSeverity::Warning,
                    );
                    return;
                }
                match crypto::create_lock(&self.new_project_password) {
                    Ok(li) => {
                        if let Ok(dir) = config::global_dir() {
                            let _ =
                                fs::write(lock_path(&dir), serde_json::to_vec_pretty(&li).unwrap());
                            self.passwords
                                .insert("global".into(), self.new_project_password.clone());
                            self.add_notification(
                                "Global project locked".to_string(),
                                NotificationSeverity::Success,
                            );
                            self.new_project_password.clear();
                            self.new_project_confirm_password.clear();
                            self.refresh_config();
                        }
                    }
                    Err(e) => self.add_notification(
                        format!("Lock failed: {}", e),
                        NotificationSeverity::Error,
                    ),
                }
            }
            (false, true) => {
                if let Ok(dir) = config::global_dir() {
                    let _ = fs::remove_file(lock_path(&dir));
                    self.passwords.remove("global");
                    self.add_notification(
                        "Global project unlocked".to_string(),
                        NotificationSeverity::Success,
                    );
                    self.refresh_config();
                    self.maps_cache.remove("global");
                }
            }
            _ => {}
        }
    }

    fn handle_master_lock_enable(&mut self) {
        if self.master_password_input.is_empty() {
            self.add_notification(
                "Enter master password".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }
        if self.master_password_input != self.master_password_confirm {
            self.add_notification(
                "Passwords do not match".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }
        if self.master_password_input.len() < 8 {
            self.add_notification(
                "Master password must be at least 8 characters".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        match crate::master_lock::enable_master_lock(&self.master_password_input) {
            Ok(_) => {
                self.master_lock_enabled = true;
                self.show_master_lock_dialog = false;
                self.master_password_input.clear();
                self.master_password_confirm.clear();
                self.add_notification(
                    "🔒 Global Master Lock ENABLED - All projects now require master password"
                        .to_string(),
                    NotificationSeverity::Success,
                );
            }
            Err(e) => self.add_notification(
                format!("Failed to enable master lock: {}", e),
                NotificationSeverity::Error,
            ),
        }
    }

    fn handle_master_lock_disable(&mut self) {
        if self.master_password_input.is_empty() {
            self.add_notification(
                "Enter master password to disable".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        match crate::master_lock::verify_master_password(&self.master_password_input) {
            Ok(true) => match crate::master_lock::disable_master_lock() {
                Ok(_) => {
                    self.master_lock_enabled = false;
                    self.show_master_lock_dialog = false;
                    self.master_password_input.clear();
                    self.master_password_confirm.clear();
                    self.add_notification(
                        "🔓 Global Master Lock DISABLED - Projects use individual settings"
                            .to_string(),
                        NotificationSeverity::Success,
                    );
                }
                Err(e) => self.add_notification(
                    format!("Failed to disable master lock: {}", e),
                    NotificationSeverity::Error,
                ),
            },
            Ok(false) => self.add_notification(
                "Invalid master password".to_string(),
                NotificationSeverity::Error,
            ),
            Err(e) => self.add_notification(
                format!("Error verifying password: {}", e),
                NotificationSeverity::Error,
            ),
        }
    }

    fn refresh_master_lock_status(&mut self) {
        self.master_lock_enabled = crate::master_lock::is_master_lock_enabled();
    }
}

#[cfg(feature = "gui")]
impl App for SafeHoldApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update statistics periodically
        self.update_statistics();

        // Clean old notifications
        self.notifications
            .retain(|n| n.timestamp.elapsed() < Duration::from_secs(5));

        // Top navigation bar
        egui::TopBottomPanel::top("top_nav").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 20.0;

                // App title with icon
                ui.label(
                    RichText::new("🔐 SafeHold")
                        .size(20.0)
                        .color(Color32::from_rgb(0, 150, 200)),
                );
                ui.separator();

                // Tab buttons
                let mut tab_button = |ui: &mut Ui, tab: TabView, label: &str, icon: &str| {
                    let is_selected = self.current_tab == tab;
                    let button_text = format!("{} {}", icon, label);
                    let button = if is_selected {
                        egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                            .fill(Color32::from_rgb(0, 120, 180))
                    } else {
                        egui::Button::new(button_text)
                    };
                    if ui.add(button).clicked() {
                        self.current_tab = tab;
                    }
                };

                tab_button(ui, TabView::Projects, "Projects", "📂");
                tab_button(ui, TabView::Credentials, "Credentials", "🔑");
                tab_button(ui, TabView::Global, "Global", "🌍");
                tab_button(ui, TabView::Export, "Export/Run", "📤");
                tab_button(ui, TabView::Settings, "Settings", "⚙️");
                tab_button(ui, TabView::Statistics, "Statistics", "📊");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Quick actions
                    if ui.button("➕ New Project").clicked() {
                        self.show_create = true;
                    }

                    if self.selected.is_some() {
                        if ui.button("➕ Add Credential").clicked() {
                            self.show_add_credential = true;
                        }
                    }
                });
            });
        });

        // Notifications
        if !self.notifications.is_empty() {
            egui::TopBottomPanel::top("notifications").show(ctx, |ui| {
                for notification in &self.notifications {
                    let (color, icon) = match notification.severity {
                        NotificationSeverity::Success => (Color32::from_rgb(0, 150, 0), "✅"),
                        NotificationSeverity::Warning => (Color32::from_rgb(200, 150, 0), "⚠️"),
                        NotificationSeverity::Error => (Color32::from_rgb(200, 0, 0), "❌"),
                        NotificationSeverity::Info => (Color32::from_rgb(0, 100, 200), "ℹ️"),
                    };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon));
                        ui.label(RichText::new(&notification.text).color(color));
                    });
                }
                ui.separator();
            });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            TabView::Projects => self.render_projects_tab(ui),
            TabView::Credentials => self.render_credentials_tab(ui),
            TabView::Global => self.render_global_tab(ui),
            TabView::Export => self.render_export_tab(ui),
            TabView::Settings => self.render_settings_tab(ui),
            TabView::Statistics => self.render_statistics_tab(ui),
        });

        // Modal dialogs
        self.render_create_project_dialog(ctx);
        self.render_add_credential_dialog(ctx);
        self.render_update_credential_dialog(ctx);
        self.render_delete_confirmation_dialog(ctx);
        self.render_global_dialogs(ctx);
        self.render_export_dialog(ctx);
        self.render_run_command_dialog(ctx);
        self.render_maintenance_dialogs(ctx);

        // Status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("📂 {} Projects", self.total_projects));
                ui.separator();
                ui.label(format!("🔑 {} Credentials", self.total_credentials));
                ui.separator();
                if !self.duplicate_keys.is_empty() {
                    ui.label(
                        RichText::new(format!("⚠️ {} Duplicate Keys", self.duplicate_keys.len()))
                            .color(Color32::from_rgb(200, 150, 0)),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(selected) = &self.selected {
                        let display_name = if selected == "global" {
                            "Global".to_string()
                        } else {
                            self.cfg
                                .sets
                                .iter()
                                .find(|s| &s.id == selected)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| selected.clone())
                        };
                        let lock_status = if self.is_locked(selected) {
                            " 🔒"
                        } else {
                            " 🔓"
                        };
                        ui.label(format!("Selected: {}{}", display_name, lock_status));
                    } else {
                        ui.label("No project selected");
                    }
                });
            });
        });
    }
}

#[cfg(feature = "gui")]
impl SafeHoldApp {
    fn render_projects_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Project Management");
            ui.separator();

            // Search/filter
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.search_filter);
                if ui.button("Clear").clicked() {
                    self.search_filter.clear();
                }
            });
            ui.separator();

            // Global project section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("🌍 Global Project");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("📋 View Credentials").clicked() {
                            self.select("global");
                        }

                        let is_locked = self.is_locked("global");
                        if is_locked {
                            if ui.button("🔓 Unlock Global").clicked() {
                                self.lock_unlock_global(false);
                            }
                        } else {
                            if ui.button("🔒 Lock Global").clicked() {
                                self.new_project_password.clear();
                                self.new_project_confirm_password.clear();
                            }
                        }
                    });
                });

                if self.is_locked("global") {
                    ui.horizontal(|ui| {
                        ui.label("🔒 Locked - Enter password to access:");
                        let pw = self.passwords.entry("global".into()).or_default();
                        if ui
                            .add(egui::TextEdit::singleline(pw).password(true))
                            .changed()
                        {
                            self.maps_cache.remove("global");
                        }
                        if ui.button("Unlock View").clicked() {
                            self.maps_cache.remove("global");
                            let _ = self.ensure_loaded("global");
                        }
                    });
                } else {
                    ui.label("🔓 Unlocked - Protected with app key");
                }
            });

            ui.separator();

            // Project list
            ui.heading("📂 User Projects");

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.cfg.sets.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("No projects yet. Create your first project!")
                                .size(16.0)
                                .color(Color32::GRAY),
                        );
                    });
                } else {
                    for project in &self.cfg.sets.clone() {
                        if !self.search_filter.is_empty()
                            && !project
                                .name
                                .to_lowercase()
                                .contains(&self.search_filter.to_lowercase())
                        {
                            continue;
                        }

                        ui.group(|ui| {
                            let is_locked = self.is_locked(&project.id);
                            ui.horizontal(|ui| {
                                let lock_icon = if is_locked { "🔒" } else { "🔓" };

                                ui.label(
                                    RichText::new(format!("{} {}", lock_icon, project.name))
                                        .size(16.0),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("❌ Delete").clicked() {
                                        self.delete_type = "project".to_string();
                                        self.delete_target = project.id.clone();
                                        self.delete_project = project.name.clone();
                                        self.show_delete_confirm = true;
                                    }

                                    if ui.button("📋 View Credentials").clicked() {
                                        self.select(&project.id);
                                    }

                                    if ui.button("📤 Export").clicked() {
                                        self.export_project = project.id.clone();
                                        self.show_export = true;
                                    }

                                    if ui.button("▶️ Run Command").clicked() {
                                        self.run_project = project.id.clone();
                                        self.show_run_command = true;
                                    }
                                });
                            });

                            if is_locked {
                                ui.horizontal(|ui| {
                                    ui.label("Password:");
                                    let pw = self.passwords.entry(project.id.clone()).or_default();
                                    ui.add(egui::TextEdit::singleline(pw).password(true));
                                    if ui.button("Unlock").clicked() {
                                        self.maps_cache.remove(&project.id);
                                        let _ = self.ensure_loaded(&project.id);
                                    }
                                });
                            }

                            ui.label(format!("ID: {}", project.id));
                        });
                        ui.add_space(5.0);
                    }
                }
            });
        });
    }

    fn render_credentials_tab(&mut self, ui: &mut Ui) {
        if let Some(selected) = &self.selected.clone() {
            ui.vertical(|ui| {
                let display_name = if selected == "global" {
                    "Global Project".to_string()
                } else {
                    self.cfg.sets.iter()
                        .find(|s| &s.id == selected)
                        .map(|s| format!("Project: {}", s.name))
                        .unwrap_or_else(|| format!("Project ID: {}", selected))
                };
                ui.heading(&display_name);
                ui.separator();
                // Controls
                ui.horizontal(|ui| {
                    ui.label("🔍 Filter:");
                    ui.text_edit_singleline(&mut self.search_filter);

                    ui.separator();
                    ui.checkbox(&mut self.show_passwords, "👁️ Show Values");
                    ui.separator();
                    if ui.button("➕ Add Credential").clicked() {
                        self.show_add_credential = true;
                    }

                    if ui.button("💾 Save All Changes").clicked() {
                        if let Err(e) = self.save_map(selected) {
                            self.add_notification(format!("Save failed: {}", e), NotificationSeverity::Error);
                        } else {
                            self.add_notification("All changes saved".to_string(), NotificationSeverity::Success);
                        }
                    }
                });
                ui.separator();

                // Password unlock if needed
                if self.is_locked(selected) && !self.passwords.contains_key(selected) {
                    ui.centered_and_justified(|ui| {
                        ui.group(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("🔒 This project is locked")
                                    .size(18.0)
                                    .color(Color32::from_rgb(200, 150, 0)));
                                ui.label("Enter password to view credentials:");

                                let pw = self.passwords.entry(selected.clone()).or_default();
                                ui.add(egui::TextEdit::singleline(pw)
                                    .password(true)
                                    .hint_text("Enter project password"));

                                if ui.button("🔓 Unlock").clicked() {
                                    self.maps_cache.remove(selected);
                                    let _ = self.ensure_loaded(selected);
                                }
                            });
                        });
                    });
                    return;
                }

                // Credentials table
                if let Some(map) = self.maps_cache.get(selected).cloned() {
                    if map.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("No credentials in this project yet.\nClick 'Add Credential' to get started!")
                                .size(16.0)
                                .color(Color32::GRAY));
                        });
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("credentials_grid")
                                .num_columns(4)
                                .spacing([10.0, 5.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    // Header
                                    ui.label(RichText::new("Key").strong());
                                    ui.label(RichText::new("Value").strong());
                                    ui.label(RichText::new("Type").strong());
                                    ui.label(RichText::new("Actions").strong());
                                    ui.end_row();

                                    let mut entries: Vec<(String, String)> = map.into_iter().collect();
                                    entries.sort_by(|a, b| a.0.cmp(&b.0));

                                    for (key, value) in entries {
                                        if !self.search_filter.is_empty() &&
                                           !key.to_lowercase().contains(&self.search_filter.to_lowercase()) &&
                                           !value.to_lowercase().contains(&self.search_filter.to_lowercase()) {
                                            continue;
                                        }

                                        // Key column
                                        ui.label(RichText::new(&key).monospace());

                                        // Value column
                                        let mut display_value = if self.show_passwords {
                                            value.clone()
                                        } else {
                                            if key.to_lowercase().contains("password") ||
                                               key.to_lowercase().contains("secret") ||
                                               key.to_lowercase().contains("token") ||
                                               key.to_lowercase().contains("key") {
                                                "•".repeat(value.len().min(12))
                                            } else {
                                                value.clone()
                                            }
                                        };

                                        let value_response = ui.add(
                                            egui::TextEdit::singleline(&mut display_value)
                                                .desired_width(200.0)
                                        );

                                        if value_response.changed() && self.show_passwords {
                                            if let Some(m) = self.maps_cache.get_mut(selected) {
                                                m.insert(key.clone(), display_value);
                                            }
                                        }

                                        // Type indicator
                                        let type_indicator = if key.to_lowercase().contains("password") ||
                                                              key.to_lowercase().contains("secret") ||
                                                              key.to_lowercase().contains("token") {
                                            "🔑 Secret"
                                        } else if key.to_lowercase().contains("url") ||
                                                  key.to_lowercase().contains("endpoint") {
                                            "🔗 URL"
                                        } else {
                                            "📝 Text"
                                        };
                                        ui.label(type_indicator);

                                        // Actions
                                        ui.horizontal(|ui| {
                                            if ui.small_button("📋").on_hover_text("Copy").clicked() {
                                                ui.ctx().copy_text(value.clone());
                                                self.add_notification(format!("Copied '{}'", key), NotificationSeverity::Info);
                                            }
                                            if ui.small_button("❌").on_hover_text("Delete").clicked() {
                                                self.delete_key(selected, &key);
                                            }
                                        });

                                        ui.end_row();
                                    }
                                });
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Failed to load credentials")
                            .size(16.0)
                            .color(Color32::from_rgb(200, 0, 0)));
                    });
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select a project from the Projects tab to view its credentials")
                        .size(16.0)
                        .color(Color32::GRAY),
                );
            });
        }
    }

    fn render_export_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Export/Run Commands");
            ui.separator();

            if self.cfg.sets.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("No projects available to export or run commands with")
                        .size(16.0)
                        .color(Color32::GRAY));
                });
                return;
            }

            ui.label("Select a project to export its credentials or run commands with environment variables:");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for project in &self.cfg.sets.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&project.name).size(16.0));

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("▶️ Run Command").clicked() {
                                    self.run_project = project.id.clone();
                                    self.show_run_command = true;
                                }

                                if ui.button("📤 Export").clicked() {
                                    self.export_project = project.id.clone();
                                    self.show_export = true;
                                }
                            });
                        });
                        ui.label(format!("ID: {}", project.id));

                        let is_locked = self.is_locked(&project.id);
                        if is_locked {
                            ui.label(RichText::new("🔒 Locked - Enter password in Projects tab first")
                                .color(Color32::from_rgb(200, 150, 0)));
                        } else {
                            if let Some(map) = self.maps_cache.get(&project.id) {
                                ui.label(format!("📊 {} credentials available", map.len()));
                            }
                        }
                    });
                    ui.add_space(5.0);
                }
            });
        });
    }

    fn render_settings_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Settings & Configuration");
            ui.separator();

            // General settings
            ui.group(|ui| {
                ui.label(RichText::new("General Settings").strong());
                ui.checkbox(&mut self.show_passwords, "Show password values by default");
                ui.horizontal(|ui| {
                    ui.label("Auto-save interval (seconds):");
                    ui.add(egui::DragValue::new(&mut self.auto_save_interval).range(5..=300));
                });
            });

            ui.separator();

            // Global settings
            ui.group(|ui| {
                ui.label(RichText::new("Global Project").strong());
                let is_locked = self.is_locked("global");
                if is_locked {
                    ui.horizontal(|ui| {
                        ui.label("🔒 Global project is currently locked");
                        if ui.button("🔓 Unlock").clicked() {
                            self.lock_unlock_global(false);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("🔓 Global project is unlocked");
                        ui.label("Set password to lock:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_project_password)
                            .password(true)
                            .hint_text("Enter password"));
                        ui.add(egui::TextEdit::singleline(&mut self.new_project_confirm_password)
                            .password(true)
                            .hint_text("Confirm password"));

                        let passwords_match = !self.new_project_password.is_empty() &&
                                            self.new_project_password == self.new_project_confirm_password;

                        if ui.add_enabled(passwords_match, egui::Button::new("🔒 Lock Global")).clicked() {
                            self.lock_unlock_global(true);
                            self.new_project_password.clear();
                            self.new_project_confirm_password.clear();
                        }
                    });
                }
            });

            ui.separator();

            // Master Lock Security
            ui.group(|ui| {
                ui.label(RichText::new("🔐 Global Master Lock Security").strong().color(Color32::from_rgb(220, 50, 50)));

                // Refresh master lock status
                self.refresh_master_lock_status();

                if self.master_lock_enabled {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔒 ENABLED - ALL projects require master password").color(Color32::from_rgb(220, 50, 50)));
                        if ui.button("🔓 Disable").clicked() {
                            self.master_lock_action = Some(false);
                            self.show_master_lock_dialog = true;
                        }
                    });
                    ui.label(RichText::new("⚠️ When enabled, ALL projects (including individual and global) require the same master password").small().color(Color32::GRAY));
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔓 DISABLED - Projects use individual lock settings").color(Color32::from_rgb(100, 150, 100)));
                        if ui.button("🔒 Enable").clicked() {
                            self.master_lock_action = Some(true);
                            self.show_master_lock_dialog = true;
                        }
                    });
                    ui.label(RichText::new("Enable for unified password protection across ALL projects").small().color(Color32::GRAY));
                }
            });

            ui.separator();

            // Application info
            ui.group(|ui| {
                ui.label(RichText::new("Application Information").strong());
                ui.horizontal(|ui| {
                    ui.label("SafeHold Credential Manager");
                    if ui.small_button("ℹ️ About").clicked() {
                        self.show_about_dialog = true;
                    }
                });
                ui.label(format!("Version: {} - GUI Mode 🖥️", env!("CARGO_PKG_VERSION")));
                ui.label(format!("Author: {}", env!("CARGO_PKG_AUTHORS")));
                ui.label("Built with Rust + egui");
                ui.horizontal(|ui| {
                    ui.label("Data directory:");
                    ui.monospace(config::base_dir().unwrap_or_default().display().to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("Repository:");
                    if ui.small_button("🌐 GitHub").clicked() {
                        // This would open the browser if we had web capabilities
                        self.add_notification("Repository: https://github.com/muhammad-fiaz/safehold".to_string(), NotificationSeverity::Info);
                    }
                });
            });

            ui.separator();

            // Maintenance operations
            ui.group(|ui| {
                ui.label(RichText::new("Maintenance Operations").strong().color(Color32::from_rgb(255, 140, 0)));

                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clean Cache").clicked() {
                        self.show_clean_cache_confirm = true;
                    }
                    ui.label("Remove temporary files and cache data");
                });

                ui.separator();

                // Danger zone
                ui.group(|ui| {
                    ui.label(RichText::new("⚠️ DANGER ZONE").strong().color(Color32::RED));
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("💥 Delete All Data").color(Color32::RED)).clicked() {
                            self.show_delete_all_confirm = true;
                        }
                        ui.label(RichText::new("Permanently delete ALL projects and data").color(Color32::RED));
                    });
                });
            });
        });
    }

    fn render_global_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("🌍 Global Credentials");
            ui.separator();

            // Load global credentials
            if let Ok(global_dir) = config::global_dir() {
                if let Ok(map) = self.read_env_map(&global_dir) {
                    self.global_credentials = map;
                }
            }

            // Add credential section
            ui.group(|ui| {
                ui.label(RichText::new("Add Global Credential").strong());
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.text_edit_singleline(&mut self.global_new_key);
                    ui.label("Value:");
                    ui.text_edit_singleline(&mut self.global_new_value);

                    let can_add =
                        !self.global_new_key.is_empty() && !self.global_new_value.is_empty();
                    if ui
                        .add_enabled(can_add, egui::Button::new("➕ Add"))
                        .clicked()
                    {
                        // Add global credential
                        if let Ok(global_dir) = config::global_dir() {
                            let mut map = self.global_credentials.clone();
                            map.insert(self.global_new_key.clone(), self.global_new_value.clone());
                            if let Err(e) = self.write_env_map(&global_dir, &map) {
                                self.add_notification(
                                    format!("Failed to add credential: {}", e),
                                    NotificationSeverity::Error,
                                );
                            } else {
                                self.add_notification(
                                    format!("Added global credential '{}'", self.global_new_key),
                                    NotificationSeverity::Success,
                                );
                                self.global_credentials = map;
                                self.global_new_key.clear();
                                self.global_new_value.clear();
                            }
                        }
                    }
                });
            });

            ui.separator();

            // Search and filter
            ui.horizontal(|ui| {
                ui.label("🔍 Filter:");
                ui.text_edit_singleline(&mut self.search_filter);
                if ui.small_button("Clear").clicked() {
                    self.search_filter.clear();
                }
            });

            ui.separator();

            // Global credentials list
            if self.global_credentials.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No global credentials found")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (key, value) in &self.global_credentials.clone() {
                        if !self.search_filter.is_empty()
                            && !key
                                .to_lowercase()
                                .contains(&self.search_filter.to_lowercase())
                        {
                            continue;
                        }

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(key).strong().size(14.0));
                                    let display_value = if self.show_passwords {
                                        value.clone()
                                    } else {
                                        "*".repeat(value.len().min(8))
                                    };
                                    ui.monospace(display_value);
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("❌ Delete").clicked() {
                                        self.delete_type = "global".to_string();
                                        self.delete_target = key.clone();
                                        self.show_delete_confirm = true;
                                    }
                                    if ui.button("✏️ Update").clicked() {
                                        self.global_update_key = key.clone();
                                        self.global_update_value = value.clone();
                                        self.show_update_global = true;
                                    }
                                    if ui.button("📋 Copy").clicked() {
                                        ui.ctx().copy_text(value.clone());
                                        self.add_notification(
                                            "Value copied to clipboard".to_string(),
                                            NotificationSeverity::Info,
                                        );
                                    }
                                });
                            });
                        });
                    }
                });

                ui.separator();
                ui.label(format!(
                    "📊 Total: {} global credential(s)",
                    self.global_credentials.len()
                ));
            }
        });
    }

    fn render_statistics_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Statistics & Analysis");
            ui.separator();

            // Overview
            ui.group(|ui| {
                ui.label(RichText::new("Overview").strong().size(16.0));
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("{}", self.total_projects))
                                .size(24.0)
                                .color(Color32::from_rgb(0, 150, 200)),
                        );
                        ui.label("Total Projects");
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("{}", self.total_credentials))
                                .size(24.0)
                                .color(Color32::from_rgb(0, 200, 100)),
                        );
                        ui.label("Total Credentials");
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("{}", self.duplicate_keys.len()))
                                .size(24.0)
                                .color(Color32::from_rgb(200, 150, 0)),
                        );
                        ui.label("Duplicate Keys");
                    });
                });
            });

            ui.separator();

            // Duplicate keys analysis
            if !self.duplicate_keys.is_empty() {
                ui.group(|ui| {
                    ui.label(
                        RichText::new("Duplicate Keys Analysis")
                            .strong()
                            .color(Color32::from_rgb(200, 150, 0)),
                    );
                    ui.label("The following keys appear in multiple projects:");

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for (key, projects) in &self.duplicate_keys {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(key).monospace());
                                    ui.label("→");
                                    for (i, project) in projects.iter().enumerate() {
                                        if i > 0 {
                                            ui.label(",");
                                        }
                                        let display_name = if project == "global" {
                                            "Global".to_string()
                                        } else {
                                            self.cfg
                                                .sets
                                                .iter()
                                                .find(|s| &s.id == project)
                                                .map(|s| s.name.clone())
                                                .unwrap_or_else(|| project.clone())
                                        };
                                        ui.label(display_name);
                                    }
                                });
                            }
                        });
                });
            }
        });
    }

    // Modal dialog implementations
    fn render_create_project_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_create {
            return;
        }

        egui::Window::new("Create New Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("📂 Project Name:");
                        ui.text_edit_singleline(&mut self.new_project_name);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.new_project_lock, "🔒 Lock with password");
                        if self.new_project_lock {
                            ui.separator();
                            ui.label("Password:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_project_password)
                                    .password(true)
                                    .hint_text("Enter password"),
                            );
                            ui.label("Confirm:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_project_confirm_password)
                                    .password(true)
                                    .hint_text("Confirm password"),
                            );
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let can_create = !self.new_project_name.trim().is_empty()
                            && (!self.new_project_lock
                                || (!self.new_project_password.is_empty()
                                    && self.new_project_password
                                        == self.new_project_confirm_password));

                        if ui
                            .add_enabled(can_create, egui::Button::new("✅ Create"))
                            .clicked()
                        {
                            let args = CreateArgs {
                                name: self.new_project_name.clone(),
                                lock: self.new_project_lock,
                                password: if self.new_project_lock {
                                    Some(self.new_project_password.clone())
                                } else {
                                    None
                                },
                            };
                            match store::cmd_create(args) {
                                Ok(()) => {
                                    self.add_notification(
                                        "Project created successfully".to_string(),
                                        NotificationSeverity::Success,
                                    );
                                    self.new_project_name.clear();
                                    self.new_project_password.clear();
                                    self.new_project_confirm_password.clear();
                                    self.new_project_lock = false;
                                    self.show_create = false;
                                    self.refresh_config();
                                }
                                Err(e) => {
                                    self.add_notification(
                                        format!("Create failed: {}", e),
                                        NotificationSeverity::Error,
                                    );
                                }
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_create = false;
                            self.new_project_name.clear();
                            self.new_project_password.clear();
                            self.new_project_confirm_password.clear();
                            self.new_project_lock = false;
                        }
                    });
                });
            });
    }

    fn render_add_credential_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_add_credential {
            return;
        }

        egui::Window::new("Add Credential")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔑 Key:");
                        ui.text_edit_singleline(&mut self.new_key);
                    });

                    ui.horizontal(|ui| {
                        ui.label("📝 Value:");
                        if self.new_key.to_lowercase().contains("password")
                            || self.new_key.to_lowercase().contains("secret")
                            || self.new_key.to_lowercase().contains("token")
                        {
                            ui.add(egui::TextEdit::singleline(&mut self.new_val).password(true));
                        } else {
                            ui.text_edit_singleline(&mut self.new_val);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let can_add =
                            !self.new_key.trim().is_empty() && !self.new_val.trim().is_empty();

                        if ui
                            .add_enabled(can_add, egui::Button::new("✅ Add"))
                            .clicked()
                        {
                            if let Some(selected) = &self.selected.clone() {
                                self.add_kv(selected);
                                self.new_key.clear();
                                self.new_val.clear();
                                self.show_add_credential = false;
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_add_credential = false;
                            self.new_key.clear();
                            self.new_val.clear();
                        }
                    });
                });
            });
    }

    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export {
            return;
        }

        egui::Window::new("Export Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "Export credentials from project: {}",
                        self.export_project
                    ));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.export_format)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.export_format,
                                    "env".to_string(),
                                    "Environment Variables (.env)",
                                );
                                ui.selectable_value(
                                    &mut self.export_format,
                                    "json".to_string(),
                                    "JSON",
                                );
                                ui.selectable_value(
                                    &mut self.export_format,
                                    "yaml".to_string(),
                                    "YAML",
                                );
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Output file:");
                        ui.text_edit_singleline(&mut self.export_file);
                        if ui.button("📁").clicked() {
                            self.export_file = format!("credentials.{}", self.export_format);
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("📤 Export").clicked() {
                            match envops::cmd_export(ExportArgs {
                                project: Some(self.export_project.clone()),
                                global: false,
                                file: if self.export_file.is_empty() {
                                    None
                                } else {
                                    Some(self.export_file.clone())
                                },
                                force: false,
                                temp: false,
                            }) {
                                Ok(()) => {
                                    self.add_notification(
                                        "Export completed successfully".to_string(),
                                        NotificationSeverity::Success,
                                    );
                                    self.show_export = false;
                                }
                                Err(e) => {
                                    self.add_notification(
                                        format!("Export failed: {}", e),
                                        NotificationSeverity::Error,
                                    );
                                }
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_export = false;
                        }
                    });
                });
            });
    }

    fn render_run_command_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_run_command {
            return;
        }

        egui::Window::new("Run Command")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "Run command with environment from project: {}",
                        self.run_project
                    ));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        ui.text_edit_singleline(&mut self.run_command);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("▶️ Run").clicked() {
                            match envops::cmd_run(RunArgs {
                                project: self.run_project.clone(),
                                command: self
                                    .run_command
                                    .split_whitespace()
                                    .map(|s| s.to_string())
                                    .collect(),
                                with_global: false,
                            }) {
                                Ok(()) => {
                                    self.add_notification(
                                        "Command executed".to_string(),
                                        NotificationSeverity::Success,
                                    );
                                    self.show_run_command = false;
                                }
                                Err(e) => {
                                    self.add_notification(
                                        format!("Command failed: {}", e),
                                        NotificationSeverity::Error,
                                    );
                                }
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_run_command = false;
                            self.run_command.clear();
                        }
                    });
                });
            });
    }

    fn render_update_credential_dialog(&mut self, ctx: &egui::Context) {
        if self.show_update_credential {
            egui::Window::new("Update Credential")
                .fixed_size([400.0, 200.0])
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Project:");
                            ui.label(&self.update_project);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Key:");
                            ui.text_edit_singleline(&mut self.update_key);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut self.update_value);
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("✅ Update").clicked() {
                                if let Some(selected) = &self.selected.clone() {
                                    if !self.update_key.is_empty() && !self.update_value.is_empty()
                                    {
                                        let dir = if selected == "global" {
                                            config::global_dir().unwrap()
                                        } else {
                                            config::set_dir(selected).unwrap()
                                        };

                                        let password = self.passwords.get(selected).cloned();
                                        if let Ok(mut map) =
                                            read_env_map_dir(&dir, password.as_deref())
                                        {
                                            map.insert(
                                                self.update_key.clone(),
                                                self.update_value.clone(),
                                            );
                                            if let Err(e) =
                                                write_env_map_dir(&dir, &map, password.as_deref())
                                            {
                                                self.add_notification(
                                                    format!("Failed to update credential: {}", e),
                                                    NotificationSeverity::Error,
                                                );
                                            } else {
                                                self.add_notification(
                                                    format!(
                                                        "Updated credential '{}'",
                                                        self.update_key
                                                    ),
                                                    NotificationSeverity::Success,
                                                );
                                                self.maps_cache.insert(selected.clone(), map);
                                                self.show_update_credential = false;
                                                self.update_key.clear();
                                                self.update_value.clear();
                                                self.update_project.clear();
                                            }
                                        }
                                    }
                                }
                            }

                            if ui.button("❌ Cancel").clicked() {
                                self.show_update_credential = false;
                                self.update_key.clear();
                                self.update_value.clear();
                                self.update_project.clear();
                            }
                        });
                    });
                });
        }
    }

    fn render_delete_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if self.show_delete_confirm {
            egui::Window::new("Confirm Deletion")
                .fixed_size([350.0, 150.0])
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        let message = match self.delete_type.as_str() {
                            "project" => format!("Are you sure you want to delete project '{}'?",
                                if self.delete_project.is_empty() { &self.delete_target } else { &self.delete_project }),
                            "credential" => format!("Are you sure you want to delete credential '{}'?", self.delete_target),
                            "global" => format!("Are you sure you want to delete global credential '{}'?", self.delete_target),
                            _ => "Are you sure you want to delete this item?".to_string(),
                        };

                        ui.label(RichText::new(message).color(Color32::from_rgb(200, 100, 0)));
                        ui.label("This action cannot be undone.");

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("🗑️ Delete").clicked() {
                                match self.delete_type.as_str() {
                                    "project" => {
                                        if let Ok(project_dir) = config::set_dir(&self.delete_target) {
                                            // Load current config to update it
                                            if let Ok(mut cfg) = config::load_config() {
                                                let mut deleted_successfully = false;

                                                if project_dir.exists() {
                                                    if let Err(e) = fs::remove_dir_all(&project_dir) {
                                                        self.add_notification(format!("Failed to delete project: {}", e), NotificationSeverity::Error);
                                                    } else {
                                                        deleted_successfully = true;
                                                    }
                                                } else {
                                                    // Directory doesn't exist, still consider it deleted
                                                    deleted_successfully = true;
                                                }

                                                if deleted_successfully {
                                                    // Remove from config if it's not global
                                                    if self.delete_target != "global" {
                                                        cfg.sets.retain(|s| s.id != self.delete_target && s.name != self.delete_target);
                                                    } else {
                                                        cfg.global_locked = false;
                                                    }

                                                    // Save updated config
                                                    if let Err(e) = config::save_config(&cfg) {
                                                        self.add_notification(format!("Failed to update config: {}", e), NotificationSeverity::Error);
                                                    } else {
                                                        let message = if project_dir.exists() {
                                                            format!("Deleted project '{}'", self.delete_target)
                                                        } else {
                                                            format!("Deleted project '{}' (no data found)", self.delete_target)
                                                        };
                                                        self.add_notification(message, NotificationSeverity::Success);
                                                        self.refresh_config();
                                                        if self.selected.as_ref() == Some(&self.delete_target) {
                                                            self.selected = None;
                                                        }
                                                    }
                                                }
                                            } else {
                                                self.add_notification("Failed to load config for deletion".to_string(), NotificationSeverity::Error);
                                            }
                                        }
                                    },
                                    "credential" => {
                                        if let Some(selected) = &self.selected.clone() {
                                            let dir = if selected == "global" {
                                                config::global_dir().unwrap()
                                            } else {
                                                config::set_dir(selected).unwrap()
                                            };

                                            let password = self.passwords.get(selected).cloned();
                                            if let Ok(mut map) = read_env_map_dir(&dir, password.as_deref()) {
                                                map.remove(&self.delete_target);
                                                if let Err(e) = write_env_map_dir(&dir, &map, password.as_deref()) {
                                                    self.add_notification(format!("Failed to delete credential: {}", e), NotificationSeverity::Error);
                                                } else {
                                                    self.add_notification(format!("Deleted credential '{}'", self.delete_target), NotificationSeverity::Success);
                                                    self.maps_cache.insert(selected.clone(), map);
                                                }
                                            }
                                        }
                                    },
                                    "global" => {
                                        if let Ok(global_dir) = config::global_dir() {
                                            if let Ok(mut map) = read_env_map_dir(&global_dir, None) {
                                                map.remove(&self.delete_target);
                                                if let Err(e) = write_env_map_dir(&global_dir, &map, None) {
                                                    self.add_notification(format!("Failed to delete global credential: {}", e), NotificationSeverity::Error);
                                                } else {
                                                    self.add_notification(format!("Deleted global credential '{}'", self.delete_target), NotificationSeverity::Success);
                                                    self.global_credentials = map;
                                                }
                                            }
                                        }
                                    },
                                    _ => {}
                                }

                                self.show_delete_confirm = false;
                                self.delete_target.clear();
                                self.delete_type.clear();
                            }

                            if ui.button("❌ Cancel").clicked() {
                                self.show_delete_confirm = false;
                                self.delete_target.clear();
                                self.delete_type.clear();
                            }
                        });
                    });
                });
        }
    }

    fn render_global_dialogs(&mut self, ctx: &egui::Context) {
        // Global add dialog
        if self.show_add_global {
            egui::Window::new("Add Global Credential")
                .fixed_size([400.0, 180.0])
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Key:");
                            ui.text_edit_singleline(&mut self.global_new_key);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut self.global_new_value);
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            let can_add = !self.global_new_key.is_empty()
                                && !self.global_new_value.is_empty();
                            if ui
                                .add_enabled(can_add, egui::Button::new("✅ Add"))
                                .clicked()
                            {
                                if let Ok(global_dir) = config::global_dir() {
                                    if let Ok(mut map) = read_env_map_dir(&global_dir, None) {
                                        map.insert(
                                            self.global_new_key.clone(),
                                            self.global_new_value.clone(),
                                        );
                                        if let Err(e) = write_env_map_dir(&global_dir, &map, None) {
                                            self.add_notification(
                                                format!("Failed to add global credential: {}", e),
                                                NotificationSeverity::Error,
                                            );
                                        } else {
                                            self.add_notification(
                                                format!(
                                                    "Added global credential '{}'",
                                                    self.global_new_key
                                                ),
                                                NotificationSeverity::Success,
                                            );
                                            self.global_credentials = map;
                                            self.show_add_global = false;
                                            self.global_new_key.clear();
                                            self.global_new_value.clear();
                                        }
                                    }
                                }
                            }

                            if ui.button("❌ Cancel").clicked() {
                                self.show_add_global = false;
                                self.global_new_key.clear();
                                self.global_new_value.clear();
                            }
                        });
                    });
                });
        }

        // Global update dialog
        if self.show_update_global {
            egui::Window::new("Update Global Credential")
                .fixed_size([400.0, 180.0])
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Key:");
                            ui.label(&self.global_update_key);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut self.global_update_value);
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("✅ Update").clicked() {
                                if !self.global_update_value.is_empty() {
                                    if let Ok(global_dir) = config::global_dir() {
                                        if let Ok(mut map) = read_env_map_dir(&global_dir, None) {
                                            map.insert(
                                                self.global_update_key.clone(),
                                                self.global_update_value.clone(),
                                            );
                                            if let Err(e) =
                                                write_env_map_dir(&global_dir, &map, None)
                                            {
                                                self.add_notification(
                                                    format!(
                                                        "Failed to update global credential: {}",
                                                        e
                                                    ),
                                                    NotificationSeverity::Error,
                                                );
                                            } else {
                                                self.add_notification(
                                                    format!(
                                                        "Updated global credential '{}'",
                                                        self.global_update_key
                                                    ),
                                                    NotificationSeverity::Success,
                                                );
                                                self.global_credentials = map;
                                                self.show_update_global = false;
                                                self.global_update_key.clear();
                                                self.global_update_value.clear();
                                            }
                                        }
                                    }
                                }
                            }

                            if ui.button("❌ Cancel").clicked() {
                                self.show_update_global = false;
                                self.global_update_key.clear();
                                self.global_update_value.clear();
                            }
                        });
                    });
                });
        }
    }

    fn render_maintenance_dialogs(&mut self, ctx: &egui::Context) {
        // About Dialog
        if self.show_about_dialog {
            egui::Window::new("About SafeHold")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("🔐 SafeHold - Professional Credential Manager");
                        ui.separator();
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Application Information").strong());
                        ui.label(format!(
                            "Version: {} - GUI Mode 🖥️",
                            env!("CARGO_PKG_VERSION")
                        ));
                        ui.label(format!("Name: {}", env!("CARGO_PKG_NAME")));
                        ui.label(format!("Description: {}", env!("CARGO_PKG_DESCRIPTION")));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Developer Information").strong());
                        ui.label(format!("Author: {}", env!("CARGO_PKG_AUTHORS")));
                        ui.label(format!("Repository: {}", env!("CARGO_PKG_REPOSITORY")));
                        ui.label(format!("Homepage: {}", env!("CARGO_PKG_HOMEPAGE")));
                        ui.label(format!("License: {}", env!("CARGO_PKG_LICENSE")));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Security Features").strong());
                        ui.label("• AES-256-GCM encryption for data at rest");
                        ui.label("• Argon2id password hashing with salt");
                        ui.label("• Memory-safe Rust implementation");
                        ui.label("• Cross-platform secure key derivation");
                        ui.label("• No plaintext storage of sensitive data");
                    });

                    ui.horizontal(|ui| {
                        if ui.button("✅ Close").clicked() {
                            self.show_about_dialog = false;
                        }
                    });
                });
        }

        // Clean Cache Confirmation Dialog
        if self.show_clean_cache_confirm {
            egui::Window::new("Clean Cache Confirmation")
                .collapsible(false)
                .resizable(false)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("🗑️ Clean Cache and Temporary Files").heading());
                        ui.separator();
                    });

                    ui.group(|ui| {
                        ui.label("This will remove:");
                        ui.label("• Cache files and temporary data");
                        ui.label("• Build artifacts and logs");
                        ui.label("• Downloaded temporary files");
                        ui.separator();
                        ui.label(
                            RichText::new("⚠️ Your credentials will NOT be affected")
                                .strong()
                                .color(Color32::from_rgb(0, 150, 0)),
                        );
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("🗑️ Clean Cache")
                                    .color(Color32::from_rgb(255, 140, 0)),
                            )
                            .clicked()
                        {
                            match envops::cmd_clean_cache(true) {
                                Ok(()) => self.add_notification(
                                    "✅ Cache cleaned successfully".to_string(),
                                    NotificationSeverity::Success,
                                ),
                                Err(e) => self.add_notification(
                                    format!("❌ Failed to clean cache: {}", e),
                                    NotificationSeverity::Error,
                                ),
                            }
                            self.show_clean_cache_confirm = false;
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_clean_cache_confirm = false;
                        }
                    });
                });
        }

        // Delete All Confirmation Dialog
        if self.show_delete_all_confirm {
            egui::Window::new("⚠️ DANGER: Delete All Data")
                .collapsible(false)
                .resizable(false)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("🚨 PERMANENT DATA DELETION")
                                .heading()
                                .color(Color32::RED),
                        );
                        ui.separator();
                    });

                    ui.group(|ui| {
                        ui.label(
                            RichText::new("THIS WILL PERMANENTLY DELETE:")
                                .strong()
                                .color(Color32::RED),
                        );
                        ui.label(
                            RichText::new("• All credential projects and their encrypted data")
                                .color(Color32::RED),
                        );
                        ui.label(RichText::new("• Global credentials").color(Color32::RED));
                        ui.label(RichText::new("• Configuration files").color(Color32::RED));
                        ui.label(RichText::new("• Cache and temporary files").color(Color32::RED));
                        ui.separator();
                        ui.label(
                            RichText::new("⚠️ THIS ACTION CANNOT BE UNDONE!")
                                .strong()
                                .color(Color32::RED),
                        );
                        ui.label(
                            RichText::new("⚠️ ALL YOUR CREDENTIALS WILL BE LOST!")
                                .strong()
                                .color(Color32::RED),
                        );
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("💥 DELETE ALL DATA").color(Color32::RED))
                            .clicked()
                        {
                            match envops::cmd_delete_all(true) {
                                Ok(()) => {
                                    self.add_notification(
                                        "💥 All data has been permanently deleted".to_string(),
                                        NotificationSeverity::Success,
                                    );
                                    // Clear all UI state since data is gone
                                    self.cfg = Config::default();
                                    self.maps_cache.clear();
                                    self.passwords.clear();
                                    self.selected = None;
                                    self.current_tab = TabView::Projects;
                                }
                                Err(e) => self.add_notification(
                                    format!("❌ Failed to delete data: {}", e),
                                    NotificationSeverity::Error,
                                ),
                            }
                            self.show_delete_all_confirm = false;
                        }

                        if ui.button("❌ Cancel (Keep Data Safe)").clicked() {
                            self.show_delete_all_confirm = false;
                        }
                    });
                });
        }

        // Master Lock Dialog
        if self.show_master_lock_dialog {
            let window_title = match self.master_lock_action {
                Some(true) => "🔒 Enable Global Master Lock",
                Some(false) => "🔓 Disable Global Master Lock",
                None => "🔐 Master Lock Status",
            };

            egui::Window::new(window_title)
                .collapsible(false)
                .resizable(false)
                .default_width(500.0)
                .show(ctx, |ui| {
                    match self.master_lock_action {
                        Some(true) => {
                            // Enable master lock
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("🔒 Enable Global Master Lock").heading());
                                ui.separator();
                            });

                            ui.group(|ui| {
                                ui.label(RichText::new("When enabled:").strong());
                                ui.label("• ALL projects will require the SAME master password");
                                ui.label("• Individual project passwords will be IGNORED");
                                ui.label("• Provides unified security across all credentials");
                                ui.separator();
                                ui.label(
                                    RichText::new(
                                        "⚠️ This will change how ALL your projects are accessed",
                                    )
                                    .color(Color32::from_rgb(255, 140, 0)),
                                );
                            });

                            ui.group(|ui| {
                                ui.label(RichText::new("Create Master Password").strong());
                                ui.label("Password must be at least 8 characters:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.master_password_input)
                                        .password(true)
                                        .hint_text("Enter master password"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.master_password_confirm)
                                        .password(true)
                                        .hint_text("Confirm master password"),
                                );
                            });

                            ui.horizontal(|ui| {
                                let passwords_match = !self.master_password_input.is_empty()
                                    && self.master_password_input == self.master_password_confirm
                                    && self.master_password_input.len() >= 8;

                                if ui
                                    .add_enabled(
                                        passwords_match,
                                        egui::Button::new("🔒 Enable Master Lock"),
                                    )
                                    .clicked()
                                {
                                    self.handle_master_lock_enable();
                                }

                                if ui.button("❌ Cancel").clicked() {
                                    self.show_master_lock_dialog = false;
                                    self.master_password_input.clear();
                                    self.master_password_confirm.clear();
                                    self.master_lock_action = None;
                                }
                            });
                        }
                        Some(false) => {
                            // Disable master lock
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("🔓 Disable Global Master Lock").heading());
                                ui.separator();
                            });

                            ui.group(|ui| {
                                ui.label(RichText::new("When disabled:").strong());
                                ui.label("• Projects will use their individual lock settings");
                                ui.label("• Each project can have its own password");
                                ui.label("• Standard per-project security model");
                            });

                            ui.group(|ui| {
                                ui.label(RichText::new("Verify Master Password").strong());
                                ui.label("Enter your current master password to disable:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.master_password_input)
                                        .password(true)
                                        .hint_text("Enter current master password"),
                                );
                            });

                            ui.horizontal(|ui| {
                                let can_disable = !self.master_password_input.is_empty();

                                if ui
                                    .add_enabled(
                                        can_disable,
                                        egui::Button::new("🔓 Disable Master Lock"),
                                    )
                                    .clicked()
                                {
                                    self.handle_master_lock_disable();
                                }

                                if ui.button("❌ Cancel").clicked() {
                                    self.show_master_lock_dialog = false;
                                    self.master_password_input.clear();
                                    self.master_lock_action = None;
                                }
                            });
                        }
                        None => {
                            // Status display (shouldn't happen through UI, but just in case)
                            ui.label("Master Lock Status");
                            if ui.button("Close").clicked() {
                                self.show_master_lock_dialog = false;
                            }
                        }
                    }
                });
        }
    }

    /// Read environment map from a directory using GUI password handling
    fn read_env_map(&self, dir: &PathBuf) -> Result<BTreeMap<String, String>> {
        read_env_map_dir(dir, None)
    }

    /// Write environment map to a directory using GUI password handling
    fn write_env_map(&self, dir: &PathBuf, map: &BTreeMap<String, String>) -> Result<()> {
        write_env_map_dir(dir, map, None)
    }
}

#[cfg(feature = "gui")]
pub fn launch_gui() -> anyhow::Result<()> {
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    let result = eframe::run_native(
        "SafeHold - Secure Credential Manager",
        options,
        Box::new(|cc| {
            let app = SafeHoldApp::new();
            app.setup_style(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(anyhow!("GUI error: {}", e)),
    }
}

// Helper functions for GUI to read/write maps using optional password input
#[cfg(feature = "gui")]
fn load_key_for_dir_gui(dir: &PathBuf, password_opt: Option<&str>) -> Result<[u8; 32]> {
    let base = config::base_dir()?;
    let lp = lock_path(dir);
    if lp.exists() {
        let lock: LockInfo = serde_json::from_slice(&fs::read(&lp)?)?;
        let password = password_opt.ok_or_else(|| anyhow!("password-required"))?;
        let key = crypto::derive_key_from_password(password, &lock)?;
        Ok(key)
    } else {
        crypto::load_app_key(&base)
    }
}

#[cfg(feature = "gui")]
fn read_env_map_dir(dir: &PathBuf, password_opt: Option<&str>) -> Result<BTreeMap<String, String>> {
    let key = load_key_for_dir_gui(dir, password_opt)?;
    let enc = fs::read(env_enc_path(dir)).unwrap_or_default();
    if enc.is_empty() {
        return Ok(BTreeMap::new());
    }
    let pt = crypto::decrypt_with_key(&key, &enc)?;
    // Parse dotenv lines
    let s = String::from_utf8_lossy(&pt);
    let mut map = BTreeMap::new();
    for line in s.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().into(), v.trim().into());
        }
    }
    Ok(map)
}

#[cfg(feature = "gui")]
fn write_env_map_dir(
    dir: &PathBuf,
    map: &BTreeMap<String, String>,
    password_opt: Option<&str>,
) -> Result<()> {
    let key = load_key_for_dir_gui(dir, password_opt)?;
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(&format!("{}={}\n", k, v));
    }
    let ct = crypto::encrypt_with_key(&key, out.as_bytes())?;
    fs::write(env_enc_path(dir), ct)
        .with_context(|| format!("write {}", env_enc_path(dir).display()))?;
    Ok(())
}
