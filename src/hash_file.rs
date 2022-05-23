use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use hex;
use md5::Digest;

use crate::log::{self, Errors};
use crate::target_file::TargetFile;

/// 出力フォルダを作成する。
pub fn ensure_output_folder(output_folder: &Path) -> Result<(), Errors> {
    match fs::create_dir_all(output_folder) {
        Ok(_) => Ok(()),
        Err(error) => Err(log::make_error!(
            "出力フォルダを作成できませんでした。: {}",
            output_folder.to_str().unwrap()
        )
        .with(&error)
        .as_errors()),
    }
}

/// ハッシュファイルを読み込んでハッシュ情報マップを作成する。
pub fn load_hash_info(hash_filepath: &Path) -> Result<HashMap<PathBuf, Digest>, Errors> {
    // ハッシュファイルがなければ空のマップを返す
    if !hash_filepath.is_file() {
        return Ok(HashMap::with_capacity(0));
    }

    let hash_file_bytes = read_hash_file(hash_filepath)?;
    let hash_file_contents = decode_hash_file_contents(hash_file_bytes)?;

    let mut hash_info_map = HashMap::new();
    for (i, line) in hash_file_contents.lines().enumerate() {
        let (target_filepath, hash) =
            log::with_line_number(parse_hash_file_line(line), hash_filepath, i + 1)?;
        hash_info_map.insert(target_filepath, hash);
    }

    Ok(hash_info_map)
}

/// ハッシュファイルを読み込む
fn read_hash_file(hash_filepath: &Path) -> Result<Vec<u8>, Errors> {
    match fs::read(hash_filepath) {
        Ok(hash_file_bytes) => Ok(hash_file_bytes),
        Err(error) => Err(log::make_error!(
            "ハッシュファイルが読み込めませんでした。: {}",
            hash_filepath.to_str().unwrap()
        )
        .with(&error)
        .as_errors()),
    }
}

/// ハッシュファイルの内容をUTF-8でデコードする。
fn decode_hash_file_contents(hash_file_bytes: Vec<u8>) -> Result<String, Errors> {
    match String::from_utf8(hash_file_bytes) {
        Ok(hash_file_contents) => Ok(hash_file_contents),
        Err(error) => Err(
            log::make_error!("ハッシュファイルのエンコーディングが不正です。")
                .with(&error)
                .as_errors(),
        ),
    }
}

/// ハッシュファイルの行をパースする。
fn parse_hash_file_line(line: &str) -> Result<(PathBuf, Digest), Errors> {
    let (target_filepath, hash) = get_filepath_and_hash(line)?;
    let target_filepath = PathBuf::from(target_filepath);
    let hash = decode_hash(hash)?;

    Ok((target_filepath, hash))
}

/// ハッシュファイルの行から対象ファイルとハッシュを抽出する。
fn get_filepath_and_hash(line: &str) -> Result<(&str, &str), Errors> {
    match line.split_once(':') {
        Some((target_filepath, hash)) => Ok((target_filepath, hash)),
        None => Err(log::make_error!("ハッシュファイルの形式が不正です。").as_errors()),
    }
}

/// 文字列のハッシュをバイナリーに変換する。
fn decode_hash(hash: &str) -> Result<Digest, Errors> {
    match hex::decode(hash) {
        // Vec<u8>をDigestに変換する
        Ok(hash_vec) => {
            let mut hash = [0u8; 16];
            for (i, value) in hash_vec.iter().enumerate() {
                hash[i] = *value;
            }
            let hash = Digest(hash);
            Ok(hash)
        }
        Err(_) => Err(log::make_error!("ハッシュファイルの形式が不正です。").as_errors()),
    }
}

/// ハッシュファイルをバックアップする。
pub fn backup(hash_filepath: &Path) -> Result<Option<PathBuf>, Errors> {
    if !hash_filepath.is_file() {
        return Ok(None);
    }

    let backup_filepath = hash_filepath.join(".backup");
    match fs::copy(hash_filepath, backup_filepath.as_path()) {
        Ok(_) => Ok(Some(backup_filepath)),
        Err(error) => Err(
            log::make_error!("ハッシュファイルのバックアップに失敗しました。")
                .with(&error)
                .as_errors(),
        ),
    }
}

/// ハッシュ情報マップから対象ファイル一覧に存在しないファイルの情報を削除する。
pub fn remove_hash_info_for_missing_file(
    mut hash_info_map: HashMap<PathBuf, Digest>,
    target_files: &Vec<TargetFile>,
) -> HashMap<PathBuf, Digest> {
    let mut exist_keys = HashSet::with_capacity(hash_info_map.len());
    for target_file in target_files {
        if hash_info_map.contains_key(target_file.normalized_path()) {
            exist_keys.insert(target_file.normalized_path().to_path_buf());
        }
    }

    let mut remove_keys = HashSet::new();
    for target_filepath in hash_info_map.keys() {
        let target_filepath = PathBuf::from(target_filepath);
        if !exist_keys.contains(&target_filepath) {
            remove_keys.insert(target_filepath);
        }
    }

    for remove_key in remove_keys {
        hash_info_map.remove(&remove_key);
    }

    hash_info_map
}

/// 計算済みのハッシュをファイルに出力する。
pub fn write_calculated_hash(
    hash_filepath: &Path,
    hash_info_map: HashMap<PathBuf, Digest>,
) -> Result<(), Errors> {
    let hash_file_contents = to_hash_file_contents(&hash_info_map);

    match fs::write(hash_filepath, &hash_file_contents) {
        Ok(_) => Ok(()),
        Err(error) => Err(log::make_error!("ハッシュファイルの作成に失敗しました")
            .with(&error)
            .as_errors()),
    }
}

/// ハッシュ情報マップをハッシュファイルの内容に変換する。
fn to_hash_file_contents(hash_info_map: &HashMap<PathBuf, Digest>) -> String {
    let mut hash_file_contents = String::new();

    for (target_filepath, hash) in hash_info_map {
        hash_file_contents = add_hash_file_line(hash_file_contents, target_filepath, hash);
    }

    hash_file_contents
}

/// バッファにハッシュ情報を1行追記する。
pub fn add_hash_file_line(mut buff: String, target_filepath: &Path, hash: &Digest) -> String {
    buff.push_str(target_filepath.to_str().unwrap());
    buff.push(':');
    buff.push_str(
        String::from_utf8(hash.to_ascii_lowercase())
            .unwrap()
            .as_str(),
    );
    buff.push('\n');

    buff
}

// /// すでにあるハッシュ情報を一時ハッシュファイルに出力する。
// pub fn write_temp_hash_file(
//     temp_hash_file_path: &Path,
//     hash_info_map: HashMap<String, Digest>,
//     &target_files: Vec<FileInfo>,
// ) -> Result<Vec<FileInfo>, Errors> {
//     let mut calc_target_files = vec![];
//

//
//     for target_file in target_files {
//         match hash_info_map.get(&target_file.normalized_path) {
//             Some(hash) => {
//                 if let Err(_) = temp_hash_file.write(&target_file.normalized_path.as_bytes()) {
//                     return ext_errors(format!(
//                         "ハッシュファイルに書き込みできません。: {}",
//                         temp_hash_file_path.to_str().unwrap()
//                     ));
//                 }
//             }
//             None => calc_target_files.push(target_file),
//         }
//     }
//
//     Ok(calc_target_files)
// }
//

/// ハッシュファイルのバックアップを削除する。
pub fn delete_backup(backup_filepath: Option<PathBuf>) {
    if let Some(backup_filepath) = backup_filepath {
        if let Err(error) = fs::remove_file(backup_filepath.as_path()) {
            log::warn("ハッシュファイルのバックアップを削除できませんでした。");
            println!("{}", error);
        }
    }
}

/// ハッシュファイルを追記モードで開く。
pub fn open_hash_file(hash_file: &Path) -> Result<File, Errors> {
    match File::options().create(true).append(true).open(hash_file) {
        Ok(file) => Ok(file),
        Err(error) => Err(log::make_error!(
            "ハッシュファイルを開けません。: {}",
            hash_file.to_str().unwrap()
        )
        .with(&error)
        .as_errors()),
    }
}
