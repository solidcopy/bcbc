use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::log::{self, Errors};

/// 起動設定
pub struct RunOptions {
    /// カレントフォルダ
    pub current_folder: PathBuf,
    // /// ホームフォルダ
    // home_folder: PathBuf,
    /// 出力フォルダ
    output_folder: PathBuf,
    /// 設定フォルダ
    config_folder: PathBuf,
    /// コマンドライン引数一覧
    args: Vec<String>,
    /// 環境変数マップ
    envs: HashMap<String, String>,
}

impl RunOptions {
    pub fn new(
        current_folder: PathBuf,
        args: Vec<String>,
        envs: HashMap<String, String>,
    ) -> Result<RunOptions, Errors> {
        let home_folder = require_env(&envs, "BCBCHOME")?;
        let home_folder = PathBuf::from(home_folder);
        let output_folder = home_folder.join("out");
        let config_folder = home_folder.join("configs");

        Ok(RunOptions {
            current_folder,
            // home_folder,
            output_folder,
            config_folder,
            args,
            envs,
        })
    }

    /// カレントフォルダを返す。
    pub fn current_folder(&self) -> &Path {
        self.current_folder.as_path()
    }

    // /// ホームフォルダを返す。
    // pub fn home_folder(&self) -> &Path {
    //     self.home_folder.as_path()
    // }

    /// 出力フォルダのパスを返す。
    pub fn output_folder(&self) -> &Path {
        self.output_folder.as_path()
    }

    /// 設定フォルダのパスを返す。
    pub fn config_folder(&self) -> &Path {
        self.config_folder.as_path()
    }

    /// コマンドライン引数一覧を返す。
    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    // /// 指定された環境変数の値を返す。
    // pub fn env(&self, env_name: &str) -> Option<&String> {
    //     match self.envs.get(env_name) {
    //         Some(env_value) => Some(&env_value),
    //         None => None,
    //     }
    // }

    /// 指定された環境変数の値を返す。
    pub fn required_env(&self, env_name: &str) -> Result<&String, Errors> {
        require_env(&self.envs, env_name)
    }
}

/// 環境変数マップから指定された環境変数を取得する。
/// 変数がない場合はエラーを返す。
fn require_env<'a>(
    envs: &'a HashMap<String, String>,
    env_name: &str,
) -> Result<&'a String, Errors> {
    match envs.get(env_name) {
        Some(env_value) => Ok(env_value),
        None => Err(log::make_error!("環境変数{}が設定されていません。", env_name).as_errors()),
    }
}
