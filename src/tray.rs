use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::{AppEvent, AppState, MenuCommand, RecordingState};
use crate::config::{HotkeyChoice, OutputMode};

const MODEL_REGISTRY: &[&str] = &["tiny", "base", "small", "medium", "large"];

struct MenuIds {
    status: MenuId,
    output_clipboard: MenuId,
    output_paste: MenuId,
    hotkey_right_alt: MenuId,
    hotkey_caps_lock: MenuId,
    current_model: MenuId,
    model_items: Vec<(String, MenuId)>,
    quit: MenuId,
}

pub struct Tray {
    _icon: TrayIcon,
    ids: MenuIds,
}

fn build_icon() -> Icon {
    // 16x16 RGBA — simple dark circle
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    let radius = 6.0f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                rgba.extend_from_slice(&[40, 40, 40, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("failed to create icon")
}

impl Tray {
    pub fn new(state: &AppState) -> Self {
        let (menu, ids) = Self::build_menu(state);
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Murmur")
            .with_icon(build_icon())
            .build()
            .expect("failed to create tray icon");
        Self { _icon: icon, ids }
    }

    fn build_menu(state: &AppState) -> (Menu, MenuIds) {
        let menu = Menu::new();

        let status_text = match &state.recording_state {
            RecordingState::Idle => "Status: Idle",
            RecordingState::Recording => "Status: Recording...",
            RecordingState::Transcribing => "Status: Transcribing...",
        };
        let status = MenuItem::new(status_text, false, None);
        let status_id = status.id().clone();
        menu.append(&status).unwrap();
        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();

        // Output mode
        let output_sub = Submenu::new("Output Mode", true);
        let is_clipboard = state.config.output_mode == OutputMode::Clipboard;
        let output_clipboard = CheckMenuItem::new("Copy to Clipboard", true, is_clipboard, None);
        let output_paste = CheckMenuItem::new("Paste at Cursor", true, !is_clipboard, None);
        let output_clipboard_id = output_clipboard.id().clone();
        let output_paste_id = output_paste.id().clone();
        output_sub.append(&output_clipboard).unwrap();
        output_sub.append(&output_paste).unwrap();
        menu.append(&output_sub).unwrap();

        // Hotkey
        let hotkey_sub = Submenu::new("Hotkey", true);
        let is_right_alt = state.config.hotkey == HotkeyChoice::RightAlt;
        let hotkey_right_alt = CheckMenuItem::new("Right Option", true, is_right_alt, None);
        let hotkey_caps_lock = CheckMenuItem::new("Caps Lock", true, !is_right_alt, None);
        let hotkey_right_alt_id = hotkey_right_alt.id().clone();
        let hotkey_caps_lock_id = hotkey_caps_lock.id().clone();
        hotkey_sub.append(&hotkey_right_alt).unwrap();
        hotkey_sub.append(&hotkey_caps_lock).unwrap();
        menu.append(&hotkey_sub).unwrap();

        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();

        // Current model
        let current_model = MenuItem::new(
            format!("Model: {}", state.config.selected_model),
            false,
            None,
        );
        let current_model_id = current_model.id().clone();
        menu.append(&current_model).unwrap();

        // Model selection
        let model_sub = Submenu::new("Change Model", true);
        let mut model_items = Vec::new();
        for &model in MODEL_REGISTRY {
            let installed = state.installed_models.contains(&model.to_string());
            let label = if installed {
                model.to_string()
            } else {
                format!("{model} (download)")
            };
            let item = MenuItem::new(label, true, None);
            model_items.push((model.to_string(), item.id().clone()));
            model_sub.append(&item).unwrap();
        }
        menu.append(&model_sub).unwrap();

        // Download progress
        if let Some((ref name, pct)) = state.download_progress {
            menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
                .unwrap();
            let progress = MenuItem::new(format!("Downloading: {name} ({pct}%)"), false, None);
            menu.append(&progress).unwrap();
        }

        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();
        let quit = MenuItem::new("Quit", true, None);
        let quit_id = quit.id().clone();
        menu.append(&quit).unwrap();

        let ids = MenuIds {
            status: status_id,
            output_clipboard: output_clipboard_id,
            output_paste: output_paste_id,
            hotkey_right_alt: hotkey_right_alt_id,
            hotkey_caps_lock: hotkey_caps_lock_id,
            current_model: current_model_id,
            model_items,
            quit: quit_id,
        };
        (menu, ids)
    }

    pub fn handle_menu_event(&self, event: &MenuEvent, state: &AppState) -> Option<AppEvent> {
        let id = &event.id;

        if *id == self.ids.quit {
            return Some(AppEvent::Quit);
        }
        if *id == self.ids.output_clipboard {
            return Some(AppEvent::Menu(MenuCommand::SetOutputMode(
                OutputMode::Clipboard,
            )));
        }
        if *id == self.ids.output_paste {
            return Some(AppEvent::Menu(MenuCommand::SetOutputMode(
                OutputMode::PasteAtCursor,
            )));
        }
        if *id == self.ids.hotkey_right_alt {
            return Some(AppEvent::Menu(MenuCommand::SetHotkey(
                HotkeyChoice::RightAlt,
            )));
        }
        if *id == self.ids.hotkey_caps_lock {
            return Some(AppEvent::Menu(MenuCommand::SetHotkey(
                HotkeyChoice::CapsLock,
            )));
        }

        for (model_name, menu_id) in &self.ids.model_items {
            if id == menu_id {
                if state.installed_models.contains(model_name) {
                    return Some(AppEvent::Menu(MenuCommand::SelectModel(model_name.clone())));
                } else {
                    return Some(AppEvent::Menu(MenuCommand::DownloadModel(
                        model_name.clone(),
                    )));
                }
            }
        }

        None
    }

    pub fn rebuild(&mut self, state: &AppState) {
        let (menu, ids) = Self::build_menu(state);
        self._icon.set_menu(Some(Box::new(menu)));
        self.ids = ids;
    }
}
