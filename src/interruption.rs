use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::log::{self, Errors};

/// Ctrl+Cハンドラを設定する。
pub fn set_interruption_handler() -> Result<Arc<AtomicBool>, Errors> {
    let interruption_flag = Arc::new(AtomicBool::new(false));
    let flag_for_handler = interruption_flag.clone();
    match ctrlc::set_handler(move || flag_for_handler.store(true, Ordering::Relaxed)) {
        Ok(_) => Ok(interruption_flag),
        Err(error) => Err(log::make_error!("Ctrl+Cハンドラが設定できませんでした。")
            .with(&error)
            .as_errors()),
    }
}
