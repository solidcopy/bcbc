use std::collections::HashMap;
use std::path::{Path, PathBuf};

use md5::Digest;
use unicode_normalization::UnicodeNormalization;

use crate::filter::Filters;

/// 対象ファイル
pub struct TargetFile {
    actual_path: PathBuf,
    normalized_path: PathBuf,
    pub size: u64,
}

impl TargetFile {
    /// インスタンスを作成する。
    pub fn new(actual_path: PathBuf, size: u64) -> TargetFile {
        let normalized_path = actual_path.to_str().unwrap().nfc().to_string();
        let normalized_path = PathBuf::from(normalized_path);

        TargetFile {
            actual_path,
            normalized_path,
            size,
        }
    }

    /// ファイルパスを返す。
    pub fn actual_path(&self) -> &Path {
        self.actual_path.as_path()
    }

    /// 正規化ファイルパスを返す。
    pub fn normalized_path(&self) -> &Path {
        self.normalized_path.as_path()
    }
}

/// 対象ファイルを一覧にする。
pub fn list_target_files(disk_root: &Path, filters: &Filters) -> Vec<TargetFile> {
    let mut target_files = vec![];
    collect_dir_entries_recursive(&mut target_files, disk_root, filters);
    target_files
}

/// 指定されたフォルダ配下のエントリーを一覧に追加する。
fn collect_dir_entries_recursive(
    target_files: &mut Vec<TargetFile>,
    folder: &Path,
    filters: &Filters,
) {
    // フォルダのエントリーをループするイテレーターを取得する
    // 取得できなければこのフォルダは処理しない
    if let Ok(dir_entry_iter) = folder.read_dir() {
        for dir_entry_result in dir_entry_iter {
            // エントリーを取得する
            // 取得できなければこのエントリーは処理しない
            if let Ok(dir_entry) = dir_entry_result {
                // フォルダなら再帰的にエントリー取得を行う
                // ファイルなら一覧に追加する
                if let Ok(metadata) = dir_entry.metadata() {
                    let dir_entry_path = dir_entry.path();
                    if metadata.is_dir() {
                        collect_dir_entries_recursive(
                            target_files,
                            dir_entry_path.as_path(),
                            filters,
                        );
                    } else if filters.is_target(dir_entry_path.as_path()) {
                        let target_file = TargetFile::new(dir_entry_path, metadata.len());
                        target_files.push(target_file);
                    }
                }
            }
        }
    }
}

/// 対象ファイルの一覧からハッシュファイルに情報があったものを除外する。
pub fn remove_calculated_file(
    target_files: Vec<TargetFile>,
    hash_info_map: &HashMap<PathBuf, Digest>,
) -> Vec<TargetFile> {
    let mut trimmed_target_files = vec![];

    for target_file in target_files {
        if !hash_info_map.contains_key(&target_file.normalized_path().to_path_buf()) {
            trimmed_target_files.push(target_file);
        }
    }

    trimmed_target_files
}

/// 対象ファイルの容量を合計する。
pub fn calc_total_size(target_files: &Vec<TargetFile>) -> u64 {
    let mut total_size = 0;
    for calc_target_file in target_files {
        total_size += calc_target_file.size;
    }
    total_size
}
