#[cfg(feature = "gui")]
use anyhow::{anyhow, Context, Result};
#[cfg(feature = "gui")]
use eframe::{egui, App};
#[cfg(feature = "gui")]
use std::collections::{BTreeMap, HashMap};
#[cfg(feature = "gui")]
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::fs;
#[cfg(feature = "gui")]
use crate::config::{self, Config};
#[cfg(feature = "gui")]
use crate::config::{env_enc_path, lock_path};
#[cfg(feature = "gui")]
use crate::crypto::{self, LockInfo};
#[cfg(feature = "gui")]
use crate::store;
#[cfg(feature = "gui")]
use crate::cli::CreateArgs;

#[cfg(feature = "gui")]
pub fn launch_gui() -> Result<()> {
    // Ensure layout exists before GUI starts
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;
    let options = eframe::NativeOptions::default();
    let result = eframe::run_native(
        "SafeHold",
        options,
        Box::new(|_cc| Ok(Box::new(SafeHoldApp::new()))),
    );
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(anyhow!("GUI error: {}", e)),
    }
}

#[cfg(feature = "gui")]
struct SafeHoldApp {
    cfg: Config,
    selected: Option<String>, // "global" or project id
    maps_cache: HashMap<String, BTreeMap<String, String>>, // project_id_or_global -> map
    passwords: HashMap<String, String>, // project_id_or_global -> password
    // create project dialog
    show_create: bool,
    new_project_name: String,
    new_project_lock: bool,
    new_project_password: String,
    // kv inputs
    filter: String,
    new_key: String,
    new_val: String,
    // message area
    message: String,
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
            show_create: false,
            new_project_name: String::new(),
            new_project_lock: false,
            new_project_password: String::new(),
            filter: String::new(),
            new_key: String::new(),
            new_val: String::new(),
            message: String::new(),
        }
    }

    fn select(&mut self, id_or_global: &str) {
        self.selected = Some(id_or_global.to_string());
        // Load map into cache lazily
        let _ = self.ensure_loaded(id_or_global);
    }

    fn ensure_loaded(&mut self, id_or_global: &str) -> Result<()> {
        if self.maps_cache.contains_key(id_or_global) { return Ok(()); }
        let dir = if id_or_global == "global" { config::global_dir()? } else { config::set_dir(id_or_global)? };
        let pwd = self.passwords.get(id_or_global).map(|s| s.as_str());
        match read_env_map_dir(&dir, pwd) {
            Ok(map) => { self.maps_cache.insert(id_or_global.to_string(), map); Ok(()) }
            Err(e) => {
                self.message = format!("Load failed: {}", e);
                Err(e)
            }
        }
    }

    fn is_locked(&self, id_or_global: &str) -> bool {
        let dir = match if id_or_global == "global" { config::global_dir() } else { config::set_dir(id_or_global) } {
            Ok(d) => d, Err(_) => return false,
        };
        lock_path(&dir).exists()
    }

    fn add_kv(&mut self, id_or_global: &str) {
        if self.new_key.trim().is_empty() { self.message = "Key cannot be empty".into(); return; }
        let map = self.maps_cache.entry(id_or_global.to_string()).or_default();
        map.insert(self.new_key.trim().to_string(), self.new_val.clone());
        if let Err(e) = self.save_map(id_or_global) { self.message = format!("Save failed: {}", e); return; }
        self.new_key.clear();
        self.new_val.clear();
        self.message = "Added".into();
    }

    fn save_map(&mut self, id_or_global: &str) -> Result<()> {
        let dir = if id_or_global == "global" { config::global_dir()? } else { config::set_dir(id_or_global)? };
        let pwd = self.passwords.get(id_or_global).map(|s| s.as_str());
        if let Some(map) = self.maps_cache.get(id_or_global) {
            write_env_map_dir(&dir, map, pwd)
        } else { Ok(()) }
    }

    fn delete_key(&mut self, id_or_global: &str, key: &str) {
        if let Some(map) = self.maps_cache.get_mut(id_or_global) {
            map.remove(key);
            if let Err(e) = self.save_map(id_or_global) { self.message = format!("Delete failed: {}", e); }
        }
    }

    fn refresh_config(&mut self) {
        if let Ok(cfg) = config::load_config() { self.cfg = cfg; }
    }

    fn lock_unlock_global(&mut self, lock: bool) {
        match (lock, self.is_locked("global")) {
            (true, false) => {
                // create lock.json
                if self.new_project_password.is_empty() {
                    self.message = "Enter password in the field before locking".into();
                    return;
                }
                match crypto::create_lock(&self.new_project_password) {
                    Ok(li) => {
                        if let Ok(dir) = config::global_dir() {
                            let _ = fs::write(lock_path(&dir), serde_json::to_vec_pretty(&li).unwrap());
                            self.passwords.insert("global".into(), self.new_project_password.clone());
                            self.message = "Global locked".into();
                            self.refresh_config();
                        }
                    }
                    Err(e) => self.message = format!("Lock failed: {}", e),
                }
            }
            (false, true) => {
                // remove lock.json
                if let Ok(dir) = config::global_dir() {
                    let _ = fs::remove_file(lock_path(&dir));
                    self.passwords.remove("global");
                    self.message = "Global unlocked".into();
                    self.refresh_config();
                    self.maps_cache.remove("global"); // force reload with app key
                }
            }
            _ => {}
        }
    }

    fn stats(&mut self) -> (usize, usize, Vec<(String, Vec<String>)>) {
        // total projects, total credentials, duplicates across projects by key name
        let total_projects = self.cfg.sets.len();
        let mut total_creds = 0usize;
        let mut occurrences: HashMap<String, Vec<String>> = HashMap::new();
        // include global as a pseudo-project named "global"
        let mut sets: Vec<(String, PathBuf)> = vec![("global".into(), config::global_dir().unwrap_or_else(|_| PathBuf::new()))];
        for s in &self.cfg.sets {
            if let Ok(dir) = config::set_dir(&s.id) { sets.push((s.id.clone(), dir)); }
        }
        for (id, dir) in sets {
            let pwd = self.passwords.get(&id).map(|s| s.as_str());
            if let Ok(map) = read_env_map_dir(&dir, pwd) {
                total_creds += map.len();
                for k in map.keys() { occurrences.entry(k.clone()).or_default().push(id.clone()); }
            }
        }
        let mut dups: Vec<(String, Vec<String>)> = occurrences.into_iter().filter_map(|(k,v)| if v.len()>1 { Some((k,v)) } else { None }).collect();
        dups.sort_by(|a,b| a.0.cmp(&b.0));
        (total_projects, total_creds, dups)
    }
}

#[cfg(feature = "gui")]
impl App for SafeHoldApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SafeHold Credential Manager");
                ui.separator();
                if ui.button("Create Project").clicked() { self.show_create = true; }
                ui.separator();
                let is_locked = self.is_locked("global");
                if is_locked {
                    if ui.button("Unlock Global").clicked() { self.lock_unlock_global(false); }
                } else {
                    let te = egui::TextEdit::singleline(&mut self.new_project_password).hint_text("Global password");
                    ui.add(te);
                    if ui.button("Lock Global").clicked() { self.lock_unlock_global(true); }
                }
            });
        });

        egui::SidePanel::left("side").resizable(true).show(ctx, |ui| {
            ui.heading("Global");
            let locked = self.is_locked("global");
            if ui.selectable_label(self.selected.as_deref()==Some("global"), if locked {"global (locked)"} else {"global"}).clicked() {
                self.select("global");
            }
            if locked {
                ui.horizontal(|ui|{
                    ui.label("Password:");
                    let pw = self.passwords.entry("global".into()).or_default();
                    ui.add(egui::TextEdit::singleline(pw).password(true));
                    if ui.button("Unlock view").clicked() { self.maps_cache.remove("global"); let _=self.ensure_loaded("global"); }
                });
            }
            ui.separator();
            ui.heading("Projects");
            for s in self.cfg.sets.clone() {
                let label = if self.is_locked(&s.id) { format!("{} (locked)", s.name) } else { s.name.clone() };
                if ui.selectable_label(self.selected.as_deref()==Some(&s.id), label).clicked() {
                    self.select(&s.id);
                }
                ui.horizontal(|ui|{
                    if self.selected.as_deref()==Some(&s.id) {
                        if ui.small_button("Delete project").clicked() {
                            if let Err(e) = store::cmd_delete_set(&s.id) { self.message = format!("Delete project failed: {}", e); }
                            self.refresh_config();
                            self.maps_cache.remove(&s.id);
                            if self.selected.as_deref()==Some(&s.id) { self.selected=None; }
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(sel) = self.selected.clone() {
                // password prompt section if locked
                if self.is_locked(&sel) {
                    ui.horizontal(|ui|{
                        ui.label("Password:");
                        let pw = self.passwords.entry(sel.clone()).or_default();
                        ui.add(egui::TextEdit::singleline(pw).password(true));
                        if ui.button("Unlock").clicked() { self.maps_cache.remove(&sel); let _=self.ensure_loaded(&sel); }
                    });
                    ui.separator();
                }
                ui.horizontal(|ui|{
                    ui.label("Filter:");
                    let te = egui::TextEdit::singleline(&mut self.filter).hint_text("filter by key");
                    ui.add(te);
                });
                ui.separator();
                // display table of key-values
                if let Some(map) = self.maps_cache.get(&sel).cloned() {
                    let entries: Vec<(String, String)> = map.into_iter().collect();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (k, v) in entries {
                            if !self.filter.is_empty() && !k.contains(&self.filter) { continue; }
                            ui.horizontal(|ui|{
                                ui.label(k.clone());
                                let mut val = v.clone();
                                let resp = ui.add(egui::TextEdit::singleline(&mut val));
                                if resp.changed() {
                                    if let Some(m) = self.maps_cache.get_mut(&sel) { m.insert(k.clone(), val.clone()); }
                                }
                                if ui.small_button("Delete").clicked() {
                                    self.delete_key(&sel, &k);
                                }
                            });
                            ui.separator();
                        }
                    });
                    if ui.button("Save all changes").clicked() { if let Err(e)=self.save_map(&sel){ self.message=format!("Save failed: {}", e);} }
                } else {
                    ui.label("No data or failed to load.");
                }
                ui.separator();
                ui.horizontal(|ui|{
                    let te_key = egui::TextEdit::singleline(&mut self.new_key).hint_text("KEY");
                    ui.add(te_key);
                    let te_val = egui::TextEdit::singleline(&mut self.new_val).hint_text("value");
                    ui.add(te_val);
                    if ui.button("Add").clicked() { self.add_kv(&sel); }
                });
            } else {
                ui.label("Select a project to view credentials.");
            }
            if !self.message.is_empty() { ui.separator(); ui.label(&self.message); }
        });

        // create project modal-like area
        if self.show_create {
            egui::Window::new("Create Project").collapsible(false).resizable(false).show(ctx, |ui|{
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_project_name);
                ui.horizontal(|ui|{
                    ui.checkbox(&mut self.new_project_lock, "Lock with password");
                    if self.new_project_lock { let te = egui::TextEdit::singleline(&mut self.new_project_password).hint_text("password"); ui.add(te); }
                });
                ui.horizontal(|ui|{
                    if ui.button("Create").clicked() {
                        let args = CreateArgs { name: self.new_project_name.clone(), lock: self.new_project_lock, password: if self.new_project_lock { Some(self.new_project_password.clone()) } else { None } };
                        match store::cmd_create(args) {
                            Ok(()) => {
                                self.message = "Project created".into();
                                // after refresh, resolve id by name to store password mapping under id
                                self.new_project_name.clear(); self.new_project_password.clear(); self.new_project_lock=false;
                                self.show_create = false;
                                self.refresh_config();
                                if self.new_project_lock {
                                    if let Some(s) = self.cfg.sets.iter().find(|s| s.name == self.new_project_name) {
                                        self.passwords.insert(s.id.clone(), self.new_project_password.clone());
                                    }
                                }
                            }
                            Err(e) => { self.message = format!("Create failed: {}", e); }
                        }
                    }
                    if ui.button("Cancel").clicked() { self.show_create = false; }
                });
            });
        }

        // stats panel bottom
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui|{
            let (total_projects, total_creds, dups) = self.stats();
            ui.horizontal(|ui|{
                ui.label(format!("Projects: {}", total_projects));
                ui.separator();
                ui.label(format!("Total credentials: {}", total_creds));
            });
            if !dups.is_empty() {
                ui.collapsing("Duplicate keys across projects", |ui|{
                    for (k, sets) in dups {
                        ui.label(format!("{} -> {:?}", k, sets));
                    }
                });
            }
        });
    }
}

// Helper functions for GUI to read/write maps using optional password input
#[cfg(feature = "gui")]
fn load_key_for_dir_gui(dir: &PathBuf, password_opt: Option<&str>) -> Result<[u8;32]> {
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
fn read_env_map_dir(dir: &PathBuf, password_opt: Option<&str>) -> Result<BTreeMap<String,String>> {
    let key = load_key_for_dir_gui(dir, password_opt)?;
    let enc = fs::read(env_enc_path(dir)).unwrap_or_default();
    if enc.is_empty() { return Ok(BTreeMap::new()); }
    let pt = crypto::decrypt_with_key(&key, &enc)?;
    // Parse dotenv lines
    let s = String::from_utf8_lossy(&pt);
    let mut map = BTreeMap::new();
    for line in s.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once('=') { map.insert(k.trim().into(), v.trim().into()); }
    }
    Ok(map)
}

#[cfg(feature = "gui")]
fn write_env_map_dir(dir: &PathBuf, map: &BTreeMap<String,String>, password_opt: Option<&str>) -> Result<()> {
    let key = load_key_for_dir_gui(dir, password_opt)?;
    let mut out = String::new();
    for (k,v) in map { out.push_str(&format!("{}={}\n", k, v)); }
    let ct = crypto::encrypt_with_key(&key, out.as_bytes())?;
    fs::write(env_enc_path(dir), ct).with_context(|| format!("write {}", env_enc_path(dir).display()))?;
    Ok(())
}
