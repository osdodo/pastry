use std::cell::OnceCell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::features::workflow_editor::types::NodeKind;
use crate::services::workflows::Workflow;
use crate::ui::language;

thread_local! {
    static MANAGER: OnceCell<GlobalHotKeyManager> = const { OnceCell::new() };
}
static LAST_HOTKEY_TIME: OnceLock<std::sync::Mutex<Instant>> = OnceLock::new();

pub const MAIN_HOTKEY_ID: u32 = 1;

pub fn setup_hotkey() -> Result<(), String> {
    let manager = GlobalHotKeyManager::new()
        .map_err(|_| language::tr(language::Text::HotkeyManagerCreateFailed).to_string())?;

    #[cfg(target_os = "macos")]
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    #[cfg(not(target_os = "macos"))]
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);

    let mut hotkey = hotkey;
    hotkey.id = MAIN_HOTKEY_ID;

    manager
        .register(hotkey)
        .map_err(|_| language::tr(language::Text::HotkeyRegisterFailed).to_string())?;

    MANAGER.with(|cell| {
        let _ = cell.set(manager);
    });

    Ok(())
}

fn with_manager(f: impl FnOnce(&GlobalHotKeyManager)) -> bool {
    MANAGER.with(|cell| {
        if let Some(manager) = cell.get() {
            f(manager);
            true
        } else {
            false
        }
    })
}

pub fn check_hotkey_event(debounce_ms: u64) -> Option<u32> {
    if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv()
        && event.state == global_hotkey::HotKeyState::Pressed
    {
        LAST_HOTKEY_TIME
            .get_or_init(|| std::sync::Mutex::new(Instant::now() - Duration::from_secs(1)));

        if let Some(last_time) = LAST_HOTKEY_TIME.get()
            && let Ok(mut time) = last_time.lock()
            && time.elapsed().as_millis() > debounce_ms as u128
        {
            *time = Instant::now();
            return Some(event.id);
        }
    }
    None
}

pub fn parse_hotkey(s: &str) -> Option<HotKey> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "cmd" | "command" | "super" => modifiers.insert(Modifiers::SUPER),
            "ctrl" | "control" => modifiers.insert(Modifiers::CONTROL),
            "alt" | "option" => modifiers.insert(Modifiers::ALT),
            "shift" => modifiers.insert(Modifiers::SHIFT),
            key_str => {
                // Try to parse as Code
                code = parse_code(key_str);
            }
        }
    }

    code.map(|c| HotKey::new(Some(modifiers), c))
}

pub fn update_workflow_hotkeys(workflows: &[Workflow]) {
    let _ = with_manager(|manager| {
        // In a real app we might want to track what's already registered to avoid redundant calls,
        // but GlobalHotKeyManager::register/unregister are relatively cheap for small numbers.

        // We don't have a list of all registered IDs to unregister, but we can just register over them or similar.
        // Actually, GlobalHotKeyManager doesn't expose what's registered.
        // However, if we register a HotKey with the same ID, it replaces.
        // But we might have deleted workflows.

        // For now, let's just register all enabled ones.
        static REGISTERED_HOTKEYS: OnceLock<std::sync::Mutex<Vec<HotKey>>> = OnceLock::new();
        let Ok(mut registered_hotkeys) = REGISTERED_HOTKEYS
            .get_or_init(|| std::sync::Mutex::new(Vec::new()))
            .lock()
        else {
            return;
        };

        // Unregister old ones
        if !registered_hotkeys.is_empty() {
            let _ = manager.unregister_all(&registered_hotkeys);
            registered_hotkeys.clear();
        }

        // Register new ones
        for workflow in workflows {
            if !workflow.enabled {
                continue;
            }

            // Find the hotkey node in the graph
            for node in &workflow.graph.nodes {
                if matches!(node.kind, NodeKind::Hotkey)
                    && let Some(combo) = &node.properties.hotkey_combo
                    && let Some(mut hotkey) = parse_hotkey(combo)
                {
                    let id = hash_id(&workflow.id);
                    hotkey.id = id;
                    if manager.register(hotkey).is_ok() {
                        registered_hotkeys.push(hotkey);
                    }
                }
            }
        }
    });
}

pub fn hash_id(id: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    id.hash(&mut s);
    // Ensure we don't collide with MAIN_HOTKEY_ID (1)
    (s.finish() % (u32::MAX as u64 - 2000)) as u32 + 2000
}

fn parse_code(s: &str) -> Option<Code> {
    match s.to_uppercase().as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "SPACE" => Some(Code::Space),
        "ENTER" => Some(Code::Enter),
        "TAB" => Some(Code::Tab),
        "ESCAPE" | "ESC" => Some(Code::Escape),
        "UP" => Some(Code::ArrowUp),
        "DOWN" => Some(Code::ArrowDown),
        "LEFT" => Some(Code::ArrowLeft),
        "RIGHT" => Some(Code::ArrowRight),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}
