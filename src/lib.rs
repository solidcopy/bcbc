use chrono::{TimeZone, Utc};
use ctrlc;
use md5;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use unicode_normalization::UnicodeNormalization;

// ファイルから1行ずつ文字列を読み込む
pub fn read_file() {
    // 全データをバイナリとして読み込む
    let s = match fs::read("sample.txt") {
        // UTF-8としてパースする
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => panic!("parse err"),
        },
        Err(_) => panic!("not found"),
    };

    for (i, line) in s.lines().enumerate() {
        assert_eq!(format!("line-{}", i + 1), line);
    }
}

// ファイルに1行ずつ文字列を書き込む
pub fn write_file() {
    let mut file = match File::create("output.txt") {
        Ok(file) => file,
        Err(_) => panic!("cannot create file"),
    };

    file.write(b"write line\n").expect("cannot write");
    file.write(b"write line\n").expect("cannot write");
    file.flush().expect("cannot flush");
}

// ファイルに追記する
pub fn append_file() {
    let mut file = match File::options().create(true).append(true).open("output.txt") {
        Ok(file) => file,
        Err(_) => panic!("cannot open file"),
    };

    let line = String::from("append line\n");
    let line = line.as_bytes();
    file.write(line).expect("cannot write");
}

// ファイルの容量を調べる
pub fn file_size() {
    let meta = match fs::metadata("sample.txt") {
        Ok(meta) => meta,
        Err(_) => panic!("cannot get size"),
    };

    let len = meta.len();
    assert_eq!(21, len);
}

// MD5を計算する
pub fn calc_md5() {
    let mut context = md5::Context::new();
    context.consume(b"abc");
    context.consume(b"xyz");
    context.consume(b"123");
    let digest = context.compute();

    assert_eq!(format!("{:?}", digest), "05d58ef1269251a11ec4d18f64d3acba");
}

// 経過時間を取得する
pub fn elapsed_time() {
    let start = Instant::now();
    thread::sleep(Duration::from_millis(2400));
    let elapsed = start.elapsed().as_millis();
    assert!((2400..2500).contains(&elapsed));
}

// 日時をフォーマットする
pub fn format_datetime() {
    let now = Utc.ymd(2022, 5, 3).and_hms(23, 55, 30);
    println!("{}", now.format("%Y-%m-%d %H:%M:%S"));
}

// 正規表現
static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?P<first>.+) of the (?P<second>.+)").unwrap());
pub fn regexp() {
    let target = "Lord of the Rings";

    assert!(RE.is_match(target));

    let captures = RE.captures(target).unwrap();
    assert_eq!(target, captures.get(0).unwrap().as_str());
    assert_eq!("Lord", captures.name("first").unwrap().as_str());
    assert_eq!("Rings", captures.name("second").unwrap().as_str());
}

// Unicode正規化
pub fn unicode_normalization() {
    let s = "パワーバランス";
    assert_eq!(7 * 3, s.nfc().to_string().len());
    assert_eq!((7 + 2) * 3, s.nfd().to_string().len());
}

// パス操作
pub fn path() {
    // ファイルのディレクトリ、上位ディレクトリを参照する
    let path = Path::new("/tmp/foo/bar.txt");
    assert_eq!(Path::new("/tmp/foo"), path.parent().unwrap());
    assert_eq!(Path::new("/tmp"), path.parent().unwrap().parent().unwrap());

    // 下位ディレクトリ、ディレクトリのファイルを参照する
    let mut path = PathBuf::from("/tmp");
    path.push("foo");
    path.push("bar.txt");
    assert_eq!("/tmp/foo/bar.txt", path.as_path().to_str().unwrap());

    // ルートの親はNone
    let root = Path::new("/");
    assert_eq!(None, root.parent());

    // ディレクトリの作成/削除
    let path = Path::new("parent-dir/sub-dir");
    fs::create_dir_all(path).unwrap();
    fs::remove_dir_all(path.parent().unwrap()).unwrap();

    // エントリの一覧
    let mut entries = fs::read_dir(".")
        .unwrap()
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.sort();

    // 参照にしないとパスの所有権がfor内に移動する
    let mut result = false;
    for e in &entries {
        if !e.is_dir() && e.file_name().unwrap().eq("sample.txt") {
            result = true;
        }
    }
    assert!(result);

    // リネーム
    fs::rename("sample.txt", "renamed.txt").unwrap();
    fs::rename("renamed.txt", "sample.txt").unwrap();
}

// Ctrl+Cのハンドリング
pub fn ctrl_c() {
    let flag = Arc::new(AtomicBool::new(true));
    let flag_for_handler = flag.clone();
    ctrlc::set_handler(move || {
        println!("Ctrl+C detected");
        flag_for_handler.store(false, Ordering::Relaxed);
    })
    .expect("ERROR Ctrl-C");

    loop {
        thread::sleep(Duration::from_secs(1));
        if flag.load(Ordering::Relaxed) {
            println!("Please Ctrl+C");
        } else {
            break;
        }
    }
}
