use image::imageops::FilterType;
use image::RgbaImage;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::{AppEvent, AppState, MenuCommand, RecordingState};
use crate::config::{HotkeyChoice, OutputMode};
use crate::languages::LANGUAGES;

// (display_name, file_name, size)
const MODEL_REGISTRY: &[(&str, &str, &str)] = &[
    ("tiny (english)", "tiny.en", "74 MB"),
    ("tiny (multilingual)", "tiny", "74 MB"),
    ("base (english)", "base.en", "141 MB"),
    ("base (multilingual)", "base", "141 MB"),
    ("small (english)", "small.en", "465 MB"),
    ("small (multilingual)", "small", "465 MB"),
    ("medium (english)", "medium.en", "1.4 GB"),
    ("medium (multilingual)", "medium", "1.4 GB"),
    ("large (multilingual)", "large-v3", "2.9 GB"),
];
const FRAME_COUNT: usize = 36;
const ICON_SIZE: u32 = 44;

struct MenuIds {
    output_clipboard: MenuId,
    output_paste: MenuId,
    hotkey_right_alt: MenuId,
    hotkey_caps_lock: MenuId,
    model_items: Vec<(String, MenuId)>,
    language_items: Vec<(String, MenuId)>,
    progress_item: Option<MenuItem>,
    website: MenuId,
    quit: MenuId,
}

pub struct Tray {
    icon: TrayIcon,
    ids: MenuIds,
    static_icon: Icon,
    frames: Vec<Icon>,
    frame_index: usize,
}

fn rotate_rgba(img: &RgbaImage, angle_rad: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w, h);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let src_x = dx * cos + dy * sin + cx;
            let src_y = -dx * sin + dy * cos + cy;

            if src_x >= 0.0 && src_x < (w - 1) as f32 && src_y >= 0.0 && src_y < (h - 1) as f32 {
                let x0 = src_x as u32;
                let y0 = src_y as u32;
                let fx = src_x - x0 as f32;
                let fy = src_y - y0 as f32;

                let p00 = img.get_pixel(x0, y0).0;
                let p10 = img.get_pixel(x0 + 1, y0).0;
                let p01 = img.get_pixel(x0, y0 + 1).0;
                let p11 = img.get_pixel(x0 + 1, y0 + 1).0;

                let mut rgba = [0u8; 4];
                for c in 0..4 {
                    let v = p00[c] as f32 * (1.0 - fx) * (1.0 - fy)
                        + p10[c] as f32 * fx * (1.0 - fy)
                        + p01[c] as f32 * (1.0 - fx) * fy
                        + p11[c] as f32 * fx * fy;
                    rgba[c] = v.round() as u8;
                }
                out.put_pixel(x, y, image::Rgba(rgba));
            }
        }
    }
    out
}

fn rgba_to_icon(img: &RgbaImage) -> Icon {
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.as_raw().clone(), w, h).expect("failed to create icon")
}

fn build_static_icon() -> (Icon, RgbaImage) {
    let png_bytes = include_bytes!("../assets/logo_light.png");
    let large = image::load_from_memory(png_bytes)
        .expect("failed to decode icon")
        .into_rgba8();
    let small = image::imageops::resize(&large, ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
    let icon = rgba_to_icon(&small);
    (icon, large)
}

fn generate_frames(large: &RgbaImage) -> Vec<Icon> {
    let mut frames = Vec::with_capacity(FRAME_COUNT);
    for i in 0..FRAME_COUNT {
        let angle = (i as f32) * std::f32::consts::TAU / FRAME_COUNT as f32;
        let rotated = rotate_rgba(large, angle);
        let small = image::imageops::resize(&rotated, ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
        frames.push(rgba_to_icon(&small));
    }
    frames
}

impl Tray {
    pub fn new(state: &AppState) -> Self {
        let (static_icon, large) = build_static_icon();
        let frames = generate_frames(&large);

        let (menu, ids) = Self::build_menu(state);
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Murmur")
            .with_icon(static_icon.clone())
            .build()
            .expect("failed to create tray icon");

        Self {
            icon,
            ids,
            static_icon,
            frames,
            frame_index: 0,
        }
    }

    pub fn advance_frame(&mut self) {
        self.frame_index = (self.frame_index + 1) % FRAME_COUNT;
        let _ = self.icon.set_icon(Some(self.frames[self.frame_index].clone()));
    }

    pub fn reset_icon(&mut self) {
        self.frame_index = 0;
        let _ = self.icon.set_icon(Some(self.static_icon.clone()));
    }

    pub fn update_progress(&self, model: &str, pct: u8) {
        if let Some(ref item) = self.ids.progress_item {
            item.set_text(format!("Downloading: {model} ({pct}%)"));
        }
    }

    fn build_menu(state: &AppState) -> (Menu, MenuIds) {
        let menu = Menu::new();

        let status_text = if !state.transcriber_ready {
            "Status: Loading model..."
        } else {
            match &state.recording_state {
                RecordingState::Idle => "Status: Idle",
                RecordingState::Recording => "Status: Recording...",
                RecordingState::Transcribing => "Status: Transcribing...",
            }
        };
        let status = MenuItem::new(status_text, false, None);
        menu.append(&status).unwrap();

        // Permission warnings (only shown when missing)
        let needs_setup = !state.permissions.microphone || !state.permissions.accessibility;
        if needs_setup {
            menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
                .unwrap();
            if !state.permissions.microphone {
                let item = MenuItem::new("⚠ Microphone: grant permission", false, None);
                menu.append(&item).unwrap();
            }
            if !state.permissions.accessibility {
                let item = MenuItem::new("⚠ Accessibility: grant permission", false, None);
                menu.append(&item).unwrap();
            }
        }

        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();

        let output_sub = Submenu::new("Output Mode", true);
        let is_clipboard = state.config.output_mode == OutputMode::Clipboard;
        let output_clipboard = CheckMenuItem::new("Copy to Clipboard", true, is_clipboard, None);
        let output_paste = CheckMenuItem::new("Paste at Cursor", true, !is_clipboard, None);
        let output_clipboard_id = output_clipboard.id().clone();
        let output_paste_id = output_paste.id().clone();
        output_sub.append(&output_clipboard).unwrap();
        output_sub.append(&output_paste).unwrap();
        menu.append(&output_sub).unwrap();

        let hotkey_sub = Submenu::new("Hotkey", true);
        let is_right_alt = state.config.hotkey == HotkeyChoice::RightAlt;
        let hotkey_right_alt = CheckMenuItem::new("Right Option", true, is_right_alt, None);
        let hotkey_caps_lock = CheckMenuItem::new("Caps Lock", true, !is_right_alt, None);
        let hotkey_right_alt_id = hotkey_right_alt.id().clone();
        let hotkey_caps_lock_id = hotkey_caps_lock.id().clone();
        hotkey_sub.append(&hotkey_right_alt).unwrap();
        hotkey_sub.append(&hotkey_caps_lock).unwrap();
        menu.append(&hotkey_sub).unwrap();

        // Languages
        let lang_sub = Submenu::new("Languages", true);
        let mut language_items = Vec::new();

        // Selected languages (toggleable)
        for lang in LANGUAGES {
            let selected = state.config.languages.contains(&lang.code.to_string()) || lang.code == "en";
            if !selected {
                continue;
            }
            let enabled = lang.code != "en";
            let item = CheckMenuItem::new(lang.name, enabled, true, None);
            language_items.push((lang.code.to_string(), item.id().clone()));
            lang_sub.append(&item).unwrap();
        }

        lang_sub.append(&tray_icon::menu::PredefinedMenuItem::separator()).unwrap();
        let add_sub = Submenu::new("Add language...", true);

        for lang in LANGUAGES {
            let selected = state.config.languages.contains(&lang.code.to_string()) || lang.code == "en";
            if selected {
                continue;
            }
            let item = CheckMenuItem::new(lang.name, true, false, None);
            language_items.push((lang.code.to_string(), item.id().clone()));
            add_sub.append(&item).unwrap();
        }
        lang_sub.append(&add_sub).unwrap();
        menu.append(&lang_sub).unwrap();

        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();

        let current_model = MenuItem::new(
            format!("Model: {}", state.config.selected_model),
            false,
            None,
        );
        menu.append(&current_model).unwrap();

        let model_sub = Submenu::new("Change Model", true);
        let mut model_items = Vec::new();
        for &(display_name, file_name, size) in MODEL_REGISTRY {
            let installed = state.installed_models.contains(&file_name.to_string());
            let label = if installed {
                format!("  {display_name}")
            } else {
                format!("⬇ {display_name}  ({size})")
            };
            let item = MenuItem::new(label, true, None);
            model_items.push((file_name.to_string(), item.id().clone()));
            model_sub.append(&item).unwrap();
        }
        menu.append(&model_sub).unwrap();

        let progress_item = if let Some((ref name, pct)) = state.download_progress {
            menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
                .unwrap();
            let progress = MenuItem::new(format!("Downloading: {name} ({pct}%)"), false, None);
            menu.append(&progress).unwrap();
            Some(progress)
        } else {
            None
        };

        menu.append(&tray_icon::menu::PredefinedMenuItem::separator())
            .unwrap();
        let version = MenuItem::new(
            format!("Murmur v{}", env!("CARGO_PKG_VERSION")),
            false,
            None,
        );
        menu.append(&version).unwrap();
        let website = MenuItem::new("Website", true, None);
        let website_id = website.id().clone();
        menu.append(&website).unwrap();
        let quit = MenuItem::new("Quit", true, None);
        let quit_id = quit.id().clone();
        menu.append(&quit).unwrap();

        let ids = MenuIds {
            output_clipboard: output_clipboard_id,
            output_paste: output_paste_id,
            hotkey_right_alt: hotkey_right_alt_id,
            hotkey_caps_lock: hotkey_caps_lock_id,
            model_items,
            language_items,
            progress_item,
            website: website_id,
            quit: quit_id,
        };
        (menu, ids)
    }

    pub fn handle_menu_event(&self, event: &MenuEvent, state: &AppState) -> Option<AppEvent> {
        let id = &event.id;

        if *id == self.ids.quit {
            return Some(AppEvent::Quit);
        }
        if *id == self.ids.website {
            let _ = std::process::Command::new("open")
                .arg("https://anubhavitis.github.io/murmur")
                .spawn();
            return None;
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

        for (lang_code, menu_id) in &self.ids.language_items {
            if id == menu_id {
                return Some(AppEvent::Menu(MenuCommand::ToggleLanguage(
                    lang_code.clone(),
                )));
            }
        }

        None
    }

    pub fn rebuild(&mut self, state: &AppState) {
        let (menu, ids) = Self::build_menu(state);
        self.icon.set_menu(Some(Box::new(menu)));
        self.ids = ids;
    }
}
