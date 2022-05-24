use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use md5::Digest;

use crate::disk::DiskInfo;
use crate::filter::Filters;
use crate::hash_file;
use crate::interruption;
use crate::log::{self, Errors};
use crate::progress::ProgressUpdate;
use crate::target_file;
use crate::target_file::TargetFile;

/// バッファサイズ
const BUFFER_SIZE: usize = 10 << 20;
/// スタックサイズ
const STACK_SIZE: usize = BUFFER_SIZE + (2 << 20);

/// ディスクごとにハッシュ計算スレッドを開始する。
pub fn start_calculation(
    disk_info_list: Vec<DiskInfo>,
    output_folder: &Path,
    filters: Filters,
    progress_tx: Sender<ProgressUpdate>,
) -> Result<HashMap<String, JoinHandle<Result<(), Errors>>>, Errors> {
    let mut worker_handles = HashMap::with_capacity(disk_info_list.len());

    for disk_info in disk_info_list {
        // マップのキーにするためコピーを取っておく
        let disk_id = disk_info.id.clone();

        let worker_handle = start_calculation_thread(
            disk_info,
            output_folder.to_path_buf(),
            filters.clone(),
            progress_tx.clone(),
        )?;

        worker_handles.insert(disk_id, worker_handle);
    }

    Ok(worker_handles)
}

/// ハッシュ計算スレッドを開始する。
fn start_calculation_thread(
    disk_info: DiskInfo,
    output_folder: PathBuf,
    filters: Filters,
    progress_tx: Sender<ProgressUpdate>,
) -> Result<JoinHandle<Result<(), Errors>>, Errors> {
    match thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || calc_procedure(disk_info, output_folder, filters, progress_tx))
    {
        Ok(handle) => Ok(handle),
        Err(error) => Err(
            log::make_error!("ハッシュ計算スレッドを開始できませんでした。")
                .with(&error)
                .as_errors(),
        ),
    }
}

/// 進捗更新メッセージを送信する。
fn send_message(
    progress_tx: &Sender<ProgressUpdate>,
    message: ProgressUpdate,
) -> Result<(), Errors> {
    match progress_tx.send(message) {
        Ok(_) => Ok(()),
        Err(error) => Err(log::make_error!("進捗更新メッセージの送信に失敗しました。")
            .with(&error)
            .as_errors()),
    }
}

/// ハッシュ計算スレッドのルーチン。
fn calc_procedure(
    disk_info: DiskInfo,
    output_folder: PathBuf,
    filters: Filters,
    progress_tx: Sender<ProgressUpdate>,
) -> Result<(), Errors> {
    // ハッシュ計算の初期処理を行う
    let (hash_filepath, target_files) =
        init_calc_procedure(&disk_info, output_folder, &filters, &progress_tx)?;

    // ハッシュファイルを追記モードで開く
    let mut hash_file = hash_file::open_hash_file(hash_filepath.as_path())?;

    // ファイル読み込み用のバッファ
    let mut buffer = [0u8; BUFFER_SIZE];

    // ファイルごとに発生したエラーの一覧
    let mut per_file_errors: Errors = vec![];

    for target_file in target_files.iter() {
        // 新規ファイル計算開始メッセージを送信する
        send_message(
            &progress_tx,
            ProgressUpdate::new_file(target_file.normalized_path().to_path_buf()),
        )?;
        // 対象ファイルを開く
        let mut file = match open_target_file(target_file.actual_path()) {
            Ok(file) => file,
            Err(errors) => {
                per_file_errors.push(errors.into_iter().next().unwrap());
                continue;
            }
        };
        // ファイルを読み込んでハッシュを計算する
        let hash = match read_and_calc_hash(&progress_tx, &mut buffer, &mut file) {
            Ok(hash) => hash,
            Err(errors) => {
                per_file_errors.push(errors.into_iter().next().unwrap());
                continue;
            }
        };
        // ハッシュファイルの行を作成する
        let hash_file_line =
            hash_file::add_hash_file_line(String::new(), target_file.normalized_path(), &hash);
        // ハッシュファイルに行を出力する
        if let Err(error) = hash_file.write(hash_file_line.as_bytes()) {
            return Err(log::make_error!("ハッシュファイルに書き込めません。")
                .with(&error)
                .as_errors());
        }
        hash_file.flush().unwrap();

        // ファイル計算完了メッセージを送信する
        send_message(&progress_tx, ProgressUpdate::done())?;
    }

    if per_file_errors.len() == 0 {
        Ok(())
    } else {
        Err(per_file_errors)
    }
}

/// ハッシュ計算の初期処理を行う。
fn init_calc_procedure(
    disk_info: &DiskInfo,
    output_folder: PathBuf,
    filters: &Filters,
    progress_tx: &Sender<ProgressUpdate>,
) -> Result<(PathBuf, Vec<TargetFile>), Errors> {
    // 初期化メッセージを送信する
    send_message(&progress_tx, ProgressUpdate::init(disk_info.id.clone()))?;
    // ハッシュファイルのパスを取得する
    let hash_filepath = output_folder.join(&disk_info.id);
    // ハッシュファイルの情報をマップにする
    let hash_info_map = hash_file::load_hash_info(hash_filepath.as_path())?;
    // ハッシュファイルをバックアップする
    let backup_filepath = hash_file::backup(hash_filepath.as_path())?;
    // 対象ファイルを一覧にする
    let target_files = target_file::list_target_files(disk_info.root_path.as_path(), &filters);
    // ハッシュ情報マップから対象ファイルが存在しない情報を削除する
    let hash_info_map = hash_file::remove_hash_info_for_missing_file(hash_info_map, &target_files);
    // 対象ファイルの一覧からハッシュファイルに情報があったものを除外する
    let target_files = target_file::remove_calculated_file(target_files, &hash_info_map);
    // 計算済みのハッシュをファイルに出力する
    hash_file::write_calculated_hash(hash_filepath.as_path(), hash_info_map)?;
    // ハッシュファイルのバックアップを削除する
    hash_file::delete_backup(backup_filepath);
    // メッセージを送信する
    let number_of_files = target_files.len();
    let total_size = target_file::calc_total_size(&target_files);
    send_message(
        &progress_tx,
        ProgressUpdate::list_targets(number_of_files, total_size),
    )?;

    Ok((hash_filepath, target_files))
}

/// 対象ファイルを開く。
fn open_target_file(target_filepath: &Path) -> Result<File, Errors> {
    match File::open(target_filepath) {
        Ok(target_file) => Ok(target_file),
        Err(error) => Err(log::make_error!("対象ファイルが開けませんでした。")
            .with(&error)
            .as_errors()),
    }
}

/// ファイルを読み込んでハッシュを計算して返す。
fn read_and_calc_hash(
    progress_tx: &Sender<ProgressUpdate>,
    mut buffer: &mut [u8],
    target_file: &mut File,
) -> Result<Digest, Errors> {
    let mut context = md5::Context::new();

    loop {
        let red_size = match target_file.read(&mut buffer) {
            Ok(red_size) => red_size,
            Err(error) => {
                return Err(log::make_error!("対象ファイルを読み込めません。")
                    .with(&error)
                    .as_errors());
            }
        };

        if red_size == 0 {
            break;
        }

        // バッファの内容をハッシュ計算に使用する
        // 配列のサイズはコンパイル時に確定している必要があるため読み込んだバイト数の配列を作れない
        // バッファがフルでない場合は1バイトずつ配列を作って渡す
        if buffer.len() == red_size {
            context.consume(&buffer);
        } else {
            for i in 0..red_size {
                context.consume([*&buffer[i]]);
            }
        }

        send_message(&progress_tx, ProgressUpdate::read(red_size as u64))?;
    }

    Ok(context.compute())
}

/// ハッシュ計算の完了を待つ。
pub fn wait_calculations(
    worker_handles: HashMap<String, JoinHandle<Result<(), Errors>>>,
) -> Result<(), Errors> {
    // Ctrl+Cハンドラを設定する
    let interruption_handler = interruption::set_interruption_handler()?;
    // スレッド終了チェック間隔
    let check_interval = Duration::from_millis(500);

    for (disk_id, worker_handle) in worker_handles {
        // 一定時間ごとにスレッドが終了しているかチェックする
        let mut finished = false;
        while !finished {
            if worker_handle.is_finished() {
                finished = true;
            } else {
                check_interruption(&interruption_handler);
                thread::sleep(check_interval);
            }
        }
        // スレッドの処理で問題が発生してもjoin自体は成功する
        if let Ok(result) = worker_handle.join() {
            if let Err(errors) = result {
                log::error(
                    format!(
                        "ディスク({}のハッシュ計算中に問題が発生しました。",
                        &disk_id
                    )
                    .as_str(),
                );
                log::log_errors(errors);
            }
        }
    }

    Ok(())
}

/// Ctrl+Cによる割り込みを受けていたらパニックを発生させる。
fn check_interruption(interruption_handler: &Arc<AtomicBool>) {
    if interruption_handler.load(Ordering::Relaxed) {
        panic!("ユーザーにより処理が停止されました。");
    }
}
