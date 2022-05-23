use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

mod calc;
mod disk;
mod filter;
mod flow;
mod hash_file;
mod interruption;
mod log;
mod merged_hash_file;
mod progress;
mod run_options;
mod target_file;

/// エントリーポイント。
fn main() {
    if let Err(errors) = execute() {
        log::log_errors(errors);
    };
}

/// 処理を実行する。
fn execute() -> Result<(), log::Errors> {
    let current_folder = get_current_folder()?;
    let args = env::args().collect();
    let envs = get_envs();
    flow::main_procedure(current_folder, args, envs)?;
    Ok(())
}

/// カレントフォルダを取得する。
fn get_current_folder() -> Result<PathBuf, log::Errors> {
    match env::current_dir() {
        Ok(current_folder) => Ok(current_folder),
        Err(_) => Err(log::make_error!("カレントフォルダが参照できません。").as_errors()),
    }
}

/// 環境変数をマップにする。
fn get_envs() -> HashMap<String, String> {
    let mut envs = HashMap::new();
    for (var_name, var_value) in env::vars() {
        envs.insert(var_name, var_value);
    }
    envs
}
