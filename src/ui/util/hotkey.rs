use iced::keyboard::{Key, Modifiers};

pub fn format_hotkey(modifiers: Modifiers, key: Key) -> Option<String> {
    // Ignore pure modifier presses
    if matches!(
        key,
        Key::Named(
            iced::keyboard::key::Named::Shift
                | iced::keyboard::key::Named::Control
                | iced::keyboard::key::Named::Alt
                | iced::keyboard::key::Named::Meta
                | iced::keyboard::key::Named::Super
        )
    ) {
        return None;
    }

    let mut parts = Vec::new();

    // Map modifiers to standard string representation
    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        if cfg!(target_os = "macos") {
            parts.push("Option");
        } else {
            parts.push("Alt");
        }
    }
    if modifiers.shift() {
        parts.push("Shift");
    }
    if modifiers.command() {
        parts.push("Cmd");
    }

    // Format the key
    let key_str = match key {
        Key::Named(named) => format!("{:?}", named),
        Key::Character(c) => c.to_uppercase().to_string(),
        Key::Unidentified => return None,
    };

    parts.push(&key_str);

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}
