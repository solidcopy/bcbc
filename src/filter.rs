use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::log::{self, Errors};
use crate::run_options::RunOptions;

/// フィルター設定
#[derive(Clone)]
pub struct Filter {
    pattern: Regex,
    inclusive: bool,
}

impl Filter {
    /// 指定されたファイルを対象とすべきか判定する。
    pub fn matches(&self, filepath: &Path) -> FilterMatch {
        match self.pattern.is_match(filepath.to_str().unwrap()) {
            true => match self.inclusive {
                true => FilterMatch::INCLUDE,
                false => FilterMatch::EXCLUDE,
            },
            false => FilterMatch::MISMATCHED,
        }
    }
}

/// フィルター一致
pub enum FilterMatch {
    INCLUDE,
    EXCLUDE,
    MISMATCHED,
}

/// フィルター設定一覧
#[derive(Clone)]
pub struct Filters {
    filters: Vec<Filter>,
}

impl Filters {
    /// 指定されたファイルがハッシュ計算の対象であるか判定する。
    pub fn is_target(&self, filepath: &Path) -> bool {
        // ファイルパスをNFCにする
        let norm_path = filepath.to_str().unwrap().nfc().to_string();
        let norm_path = Path::new(&norm_path);

        for filter in self.filters.iter() {
            match filter.matches(norm_path) {
                FilterMatch::MISMATCHED => continue,
                FilterMatch::INCLUDE => return true,
                FilterMatch::EXCLUDE => return false,
            }
        }
        // 一致するフィルターがなければ対象としない
        return false;
    }
}

/// フィルター設定一覧を作成する処理フローを実行する。
pub fn load_filters(run_options: &RunOptions) -> Result<Filters, Errors> {
    let filter_conf_file = filter_conf_filepath(run_options.config_folder());
    let filter_conf_bytes = read_filter_conf_file(filter_conf_file.as_path())?;
    let filter_conf = parse_utf8(filter_conf_bytes)?;
    let filter_conf = to_nfc(filter_conf);
    parse_filter_conf(&filter_conf)
}

/// フィルター設定ファイルのパスを返す。
fn filter_conf_filepath(config_folder: &Path) -> PathBuf {
    config_folder.join("filter.conf")
}

/// フィルター設定ファイルを読み込む。
fn read_filter_conf_file(filter_conf_file: &Path) -> Result<Vec<u8>, Errors> {
    match fs::read(filter_conf_file) {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(log::make_error!("フィルター設定ファイルが見つかりません。")
            .with(&error)
            .as_errors()),
    }
}

/// フィルター設定ファイルの内容をデコードする。
fn parse_utf8(bytes: Vec<u8>) -> Result<String, Errors> {
    match String::from_utf8(bytes) {
        Ok(contents) => Ok(contents),
        Err(error) => Err(log::make_error!(
            "フィルター設定ファイルがUTF-8のテキストファイルではありません。"
        )
        .with(&error)
        .as_errors()),
    }
}

/// フィルター設定ファイルの内容をNFCに変換する。
fn to_nfc(filter_conf: String) -> String {
    filter_conf.as_str().nfc().to_string()
}

/// フィルター設定ファイルの内容からフィルター一覧を作成する。
fn parse_filter_conf(filter_conf: &str) -> Result<Filters, Errors> {
    let mut filters: Vec<Filter> = vec![];

    let mut errors = vec![];

    // エラーメッセージに行番号を出力するためenumerateする
    for (i, line) in filter_conf.lines().enumerate() {
        match parse_filter_conf_line(line) {
            Ok(Some(filter)) => filters.push(filter),
            Ok(None) => {}
            Err(message) => {
                let error = log::make_error!(
                    "フィルター設定ファイルの形式が不正です。: {}行目: {}",
                    i + 1,
                    message
                );
                errors.push(error);
            }
        }
    }

    if errors.len() == 0 {
        Ok(Filters { filters })
    } else {
        Err(errors)
    }
}

/// フィルター設定ファイルの1行からフィルター設定を作成する。
fn parse_filter_conf_line(line: &str) -> Result<Option<Filter>, &'static str> {
    // コメント行
    if line.starts_with('#') {
        return Ok(None);
    }

    let line = line.trim();

    // +/-で始まり、続けて正規表現パターンが書かれている行ならフィルターを作成する
    let mut chars = line.chars();

    match chars.next() {
        // 最初の1文字がある
        Some(first_char) => {
            // 1文字目が + or -
            if first_char == '+' || first_char == '-' {
                let pattern = chars.collect::<String>();
                // 正規表現パターンあり
                if pattern.len() > 0 {
                    // 正規表現パターンのパースに成功
                    if let Ok(pattern) = Regex::new(&pattern) {
                        let inclusive = first_char == '+';
                        let filter = Filter { pattern, inclusive };
                        Ok(Some(filter))
                    }
                    // 正規表現パターンが不正
                    else {
                        Err("正規表現パターンが不正です。")
                    }
                }
                // 正規表現パターンなし
                else {
                    Err("正規表現パターンがありません。")
                }
            } else {
                // 1文字目がそれ以外
                Err("行頭が'+'または'-'ではありません。")
            }
        }
        // 空白行
        None => Ok(None),
    }
}
