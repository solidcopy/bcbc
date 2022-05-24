use std::collections::HashMap;
use std::path::PathBuf;

use crate::calc;
use crate::disk;
use crate::filter;
use crate::hash_file;
use crate::log::{self, Errors};
use crate::merged_hash_file;
use crate::progress;
use crate::run_options::RunOptions;

/// 主処理。
pub fn main_procedure(
    current_folder: PathBuf,
    args: Vec<String>,
    envs: HashMap<String, String>,
) -> Result<(), Errors> {
    // 起動設定を構造体に変換する
    let run_options = RunOptions::new(current_folder, args, envs)?;
    // ツール名とバージョンを出力する
    log::info(format!("bcbc v{}", run_options.required_env("CARGO_PKG_VERSION")?).as_str());
    // フィルター設定を読み込んで一覧にする
    let filters = filter::load_filters(&run_options)?;
    // ディスク情報を一覧にする
    let disk_info_list = disk::list_disk_info(run_options.current_folder(), run_options.args())?;
    // 出力フォルダの作成
    hash_file::ensure_output_folder(run_options.output_folder())?;

    log::info("ハッシュ計算を開始します。");

    // 進捗監視スレッドの開始
    let progress_tx = progress::start_progress_monitor();
    // ハッシュ計算スレッドの開始
    let worker_handles = calc::start_calculation(
        disk_info_list,
        run_options.output_folder(),
        filters,
        progress_tx,
    )?;
    // ハッシュ計算の完了を待つ
    calc::wait_calculations(worker_handles)?;
    // ハッシュファイルを統合する
    merged_hash_file::integrate_hash_files(run_options.output_folder())?;

    log::info("ハッシュ計算を終了しました。");

    Ok(())
}
