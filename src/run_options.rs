use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::log::{self, Errors};

/// 起動設定
pub struct RunOptions {
    /// カレントフォルダ
    pub current_folder: PathBuf,
    /// 出力フォルダ
    output_folder: PathBuf,
    /// 設定フォルダ
    config_folder: PathBuf,
    /// ディスクルート一覧
    disk_roots: Vec<PathBuf>,
    /// 環境変数マップ
    envs: HashMap<String, String>,
}

impl RunOptions {
    pub fn new(
        current_folder: PathBuf,
        args: Vec<String>,
        envs: HashMap<String, String>,
    ) -> Result<RunOptions, Errors> {
        // コマンドライン引数をディスクルートにパースする
        // 1つ目はこのプログラムのパス
        let mut disk_roots = Vec::with_capacity(args.len());
        for arg in args.iter().skip(1) {
            disk_roots.push(tilde_to_home(PathBuf::from(arg)));
        }
        // BCBCHOMEから各パスを求める
        let home_folder = require_env(&envs, "BCBCHOME")?;
        let home_folder = tilde_to_home(PathBuf::from(home_folder));
        let output_folder = home_folder.join("out");
        let config_folder = home_folder.join("configs");

        Ok(RunOptions {
            current_folder,
            output_folder,
            config_folder,
            disk_roots,
            envs,
        })
    }

    /// カレントフォルダを返す。
    pub fn current_folder(&self) -> &Path {
        self.current_folder.as_path()
    }

    /// 出力フォルダのパスを返す。
    pub fn output_folder(&self) -> &Path {
        self.output_folder.as_path()
    }

    /// 設定フォルダのパスを返す。
    pub fn config_folder(&self) -> &Path {
        self.config_folder.as_path()
    }

    /// ディスクルート一覧を返す。
    pub fn disk_roots(&self) -> &Vec<PathBuf> {
        &self.disk_roots
    }

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

/// 指定されたパスが"~"で始まる場合、ホームフォルダに置き換える。
fn tilde_to_home(path: PathBuf) -> PathBuf {
    if path.starts_with("~") {
        let suffix = path.strip_prefix("~").unwrap();
        let home = dirs::home_dir().unwrap();
        home.join(suffix)
    } else {
        path
    }
}
