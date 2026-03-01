mod manager;
mod qr;
mod server;
mod state;

pub const DEFAULT_WEB_SERVER_PORT: u16 = 8080;

pub use manager::{init_web_state, update_clipboard};
pub use qr::{generate_qr_svg, get_local_ip};
pub use server::{start_web_server, stop_web_server};
pub use state::{ClipboardEntry, WebState, decode_image_data_url};
