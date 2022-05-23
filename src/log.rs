use chrono::Local;
use std::fmt::{Display, Write};
use std::path::Path;

/// タイムスタンプ付きでログを出力する。
pub fn log(level: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("{} [{}] {}", timestamp, level, message);
}

/// 情報ログを出力する。
pub fn info(message: &str) {
    log("INFO", message);
}

/// 警告ログを出力する。
pub fn warn(message: &str) {
    log("WARN", message);
}

/// エラーログを出力する。
pub fn error(message: &str) {
    log("ERROR", message);
}

/// エラー情報一覧
pub type Errors = Vec<Error>;

/// エラー情報
pub struct Error {
    message: String,
    additional: Option<String>,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {
            message: String::from(message),
            additional: None,
        }
    }

    pub fn with(mut self, additional: &dyn Display) -> Self {
        let mut buff = String::new();
        write!(buff, "{}", additional).unwrap();
        self.additional = Some(buff);
        self
    }

    pub fn as_errors(self) -> Vec<Self> {
        vec![self]
    }
}

/// エラー情報を作成する。
#[macro_export]
macro_rules! make_error {
    ( $( $s:expr ),+ ) => {{
        use std::fmt::Write;
        let mut message = String::new();
        write!(message, $($s),+).unwrap();
        crate::log::Error::new(message.as_str())
    }}
}

pub(crate) use make_error;

/// エラー情報をログ出力する。
pub fn log_errors(errors: Errors) {
    for error in errors.iter() {
        log_error(error);
    }
}

/// エラー情報項目をログ出力する。
pub fn log_error(error: &Error) {
    log("ERROR", error.message.as_str());
    if let Some(additional) = &error.additional {
        println!("{}", additional);
    }
}

/// エラーメッセージにファイル名と行番号を付与する。
pub fn with_line_number<T>(
    result: Result<T, Errors>,
    filepath: &Path,
    line_number: usize,
) -> Result<T, Errors> {
    match result {
        Ok(value) => Ok(value),
        Err(mut errors) => {
            for error in errors.iter_mut() {
                error.message.push('[');
                error.message.push_str(filepath.to_str().unwrap());
                error.message.push(':');
                error.message.push_str(format!("{}", line_number).as_str());
                error.message.push(']');
            }
            Err(errors)
        }
    }
}
