use std::fs;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::{self, Error, Errors};

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub index: usize,
    pub id: String,
    pub root_path: PathBuf,
}

/// ディスクIDの正規表現パターン
pub static DISK_ID_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z]\d+$").unwrap());

/// ディスク情報一覧を作成する。
pub fn list_disk_info(
    current_folder: &Path,
    disk_roots: &Vec<PathBuf>,
) -> Result<Vec<DiskInfo>, Errors> {
    let disk_files = list_disk_files(current_folder, &disk_roots)?;

    let mut errors = Vec::<Error>::new();
    let (disk_files, missing_disk_files) = divide_disk_files_by_existence(disk_files);
    add_missing_disk_file_errors(&mut errors, &missing_disk_files);
    let (mut disk_info_list, mut load_errors) = load_disk_info_list(&disk_files);
    errors.append(&mut load_errors);
    raise_errors(errors)?;

    index_disk_info(&mut disk_info_list);
    Ok(disk_info_list)
}

/// diskファイル一覧を作成する。
fn list_disk_files(
    current_folder: &Path,
    disk_roots: &Vec<PathBuf>,
) -> Result<Vec<PathBuf>, Errors> {
    if disk_roots.len() > 0 {
        Ok(list_disk_files_by(disk_roots))
    } else {
        match find_disk_file(current_folder) {
            Some(disk_file) => Ok(vec![disk_file]),
            None => Err(log::make_error!("diskファイルがありません。").as_errors()),
        }
    }
}

/// 指定されたディスクルートのdiskファイルを一覧にする。
fn list_disk_files_by(disk_roots: &Vec<PathBuf>) -> Vec<PathBuf> {
    disk_roots
        .iter()
        .map(|disk_root| disk_root.join("disk"))
        .collect()
}

/// カレントフォルダから開始して、上位フォルダに遡りながらdiskファイルを探す。
fn find_disk_file(current_folder: &Path) -> Option<PathBuf> {
    // 編集のためコピーする
    let mut current_folder = current_folder.to_path_buf();

    loop {
        // このフォルダにdiskファイルがあればそれを返す
        let disk_file = current_folder.join("disk");
        if disk_file.is_file() {
            return Some(disk_file);
        }

        // 1つ上のフォルダに移動する
        // すでに最上位ならdiskファイルなしとする
        if !current_folder.pop() {
            return None;
        }
    }
}

/// ディスクファイル一覧の各ファイルの有無を調べて存在するファイルと存在しないファイルの一覧に分割する。
fn divide_disk_files_by_existence(disk_files: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    disk_files
        .into_iter()
        .partition(|disk_file| disk_file.is_file())
}

/// 存在しないディスクファイルについてのエラー情報を一覧に追加する。
fn add_missing_disk_file_errors(errors: &mut Vec<Error>, missing_disk_files: &Vec<PathBuf>) {
    for missing_disk_file in missing_disk_files {
        let error = log::make_error!(
            "指定されたフォルダにdiskファイルがありません。: {}",
            missing_disk_file.to_str().unwrap()
        );
        errors.push(error);
    }
}

/// diskファイルを読み込んでディスク情報一覧を作成する。
/// 読み込みに失敗したdiskファイルについてはディスク情報は作成せず、エラー情報を一覧に追加する。
fn load_disk_info_list(disk_files: &Vec<PathBuf>) -> (Vec<DiskInfo>, Vec<Error>) {
    let mut disk_info_list = vec![];
    let mut load_errors = vec![];

    for disk_file in disk_files {
        match load_disk_info(disk_file.as_path()) {
            Ok(disk_info) => disk_info_list.push(disk_info),
            Err(error) => load_errors.push(error),
        }
    }

    (disk_info_list, load_errors)
}

/// diskファイルを読み込んでディスク情報を作成する。
/// 読み込みに失敗した場合はエラー情報を返す。
fn load_disk_info(disk_file: &Path) -> Result<DiskInfo, Error> {
    // diskファイルを読み込む
    match fs::read(disk_file) {
        Ok(disk_file_bytes) => {
            // UTF-8でデコードする
            match String::from_utf8(disk_file_bytes) {
                Ok(disk_file_contents) => {
                    let disk_file_contents = disk_file_contents.trim().to_string();

                    if !DISK_ID_PATTERN.is_match(&disk_file_contents) {
                        return Err(log::make_error!(
                            "diskファイルの内容が不正です。: {}",
                            disk_file.to_str().unwrap()
                        ));
                    }

                    Ok(DiskInfo {
                        index: 0,
                        id: disk_file_contents,
                        root_path: disk_file.parent().unwrap().to_path_buf(),
                    })
                }
                Err(error) => Err(log::make_error!(
                    "diskファイルの内容が不正です。: {}",
                    disk_file.to_str().unwrap()
                )
                .with(&error)),
            }
        }
        Err(error) => Err(log::make_error!(
            "diskファイルが読み込めませんでした。: {}",
            disk_file.to_str().unwrap()
        )
        .with(&error)),
    }
}

/// エラー情報一覧が空なら何もしない。
/// 空でなければエラーを発生させる。
fn raise_errors(errors: Vec<Error>) -> Result<(), Errors> {
    if errors.len() == 0 {
        Ok(())
    } else {
        Err(errors)
    }
}

/// ディスク情報のインデックスを採番する。
fn index_disk_info(disk_info_list: &mut Vec<DiskInfo>) {
    for (index, disk_info) in disk_info_list.iter_mut().enumerate() {
        disk_info.index = index;
    }
}
