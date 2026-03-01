use std::sync::OnceLock;
use tokio::sync::RwLock;

use super::state::{ClipboardEntry, WebState};

static WEB_STATE: OnceLock<RwLock<Option<WebState>>> = OnceLock::new();

pub async fn init_web_state(state: WebState) {
    let state_lock = WEB_STATE.get_or_init(|| RwLock::new(None));
    let mut state_opt = state_lock.write().await;
    *state_opt = Some(state);
}

pub async fn update_clipboard(entry: ClipboardEntry) {
    if let Some(state_lock) = WEB_STATE.get() {
        let state_opt = state_lock.read().await;
        if let Some(state) = state_opt.as_ref() {
            state.update_clipboard(entry).await;
        }
    }
}
