use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::disk;
use crate::log::{self, Errors};

/// ハッシュファイルを統合する。
pub fn integrate_hash_files(output_folder: &Path) -> Result<(), Errors> {
    log::info("ハッシュファイルの統合を開始します。");

    // ハッシュファイルを一覧にする
    let hash_files = find_hash_files(output_folder)?;
    // ハッシュファイルをグループに分ける
    let hash_file_map = group_hash_files(hash_files);
    // 統合ハッシュファイルを出力する
    for (disk_group, hash_filepaths) in hash_file_map.iter() {
        if let Err(errors) = write_merged_hash_file(output_folder, *disk_group, hash_filepaths) {
            log::log_errors(errors);
        }
    }

    log::info("ハッシュファイルの統合を終了しました。");

    Ok(())
}

/// ハッシュファイルを一覧にする
fn find_hash_files(output_folder: &Path) -> Result<Vec<PathBuf>, Errors> {
    let mut hash_files = vec![];

    match output_folder.read_dir() {
        Ok(read_dir) => {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file()
                        && disk::DISK_ID_PATTERN
                            .is_match(path.file_name().unwrap().to_str().unwrap())
                    {
                        hash_files.push(path);
                    }
                }
            }
        }
        Err(error) => {
            return Err(
                log::make_error!("出力ファイルの一覧を取得できませんでした。")
                    .with(&error)
                    .as_errors(),
            );
        }
    }

    Ok(hash_files)
}

/// ハッシュファイルをグループに分ける。
fn group_hash_files(hash_files: Vec<PathBuf>) -> HashMap<char, Vec<PathBuf>> {
    let mut hash_file_map = HashMap::<char, Vec<PathBuf>>::new();

    for hash_file in hash_files {
        // ファイル名の1文字目
        let disk_group = hash_file.to_str().unwrap().chars().next().unwrap();
        match hash_file_map.get_mut(&disk_group) {
            Some(file_group) => file_group.push(hash_file),
            None => {
                let file_group = vec![hash_file];
                hash_file_map.insert(disk_group, file_group);
            }
        }
    }

    return hash_file_map;
}

/// 統合ハッシュファイルを出力する。
fn write_merged_hash_file(
    output_folder: &Path,
    disk_group: char,
    hash_filepaths: &Vec<PathBuf>,
) -> Result<(), Errors> {
    let merged_hash_filepath = output_folder.join(disk_group.to_string());
    let merged_hash_file_contents = merge_hash_files_contents(hash_filepaths)?;
    match fs::write(&merged_hash_filepath, &merged_hash_file_contents) {
        Ok(_) => Ok(()),
        Err(error) => Err(
            log::make_error!("統合ハッシュファイルの作成に失敗しました。")
                .with(&error)
                .as_errors(),
        ),
    }
}

/// ハッシュファイルの内容を統合する。
fn merge_hash_files_contents(hash_filepaths: &Vec<PathBuf>) -> Result<String, Errors> {
    let merged_contents = read_hash_files(hash_filepaths)?;
    let mut lines: Vec<&str> = merged_contents.lines().collect();
    lines.sort();
    let mut merged_contents = String::new();
    for line in lines {
        merged_contents.push_str(line);
    }

    Ok(merged_contents)
}

/// 指定された一覧のハッシュファイルを読み込んで内容を連結して返す。
fn read_hash_files(hash_filepaths: &Vec<PathBuf>) -> Result<String, Errors> {
    let mut merged_contents = vec![];
    let mut errors = vec![];

    for hash_filepath in hash_filepaths.iter() {
        match fs::read(hash_filepath) {
            Ok(mut contents) => merged_contents.append(&mut contents),
            Err(error) => {
                let error =
                    log::make_error!("ハッシュファイルが読み込めませんでした。").with(&error);
                errors.push(error);
            }
        }
    }

    let merged_contents = match String::from_utf8(merged_contents) {
        Ok(merged_contents) => merged_contents,
        Err(error) => {
            let error = log::make_error!("ハッシュファイルの内容が不正です。").with(&error);
            errors.push(error);
            String::new()
        }
    };

    if errors.len() == 0 {
        Ok(merged_contents)
    } else {
        Err(errors)
    }
}
