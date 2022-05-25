use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::log::{self, Errors};
use std::fmt::Write;
use std::path::PathBuf;

/// 進捗監視スレッドを開始する。
pub fn start_progress_monitor() -> Sender<ProgressUpdate> {
    let (tx, rx) = mpsc::channel::<ProgressUpdate>();
    thread::spawn(move || progress_monitor_routine(rx));
    tx
}

/// 進捗監視ルーチン。
fn progress_monitor_routine(rx: Receiver<ProgressUpdate>) -> Result<(), Errors> {
    let mut progress_summary = ProgressSummary::new();

    let mut prev_output_time = Instant::now();

    loop {
        let progress_update = receive_progress_update(&rx)?;
        let is_done = progress_update.message_type == ProgressUpdateType::Done;
        progress_summary.update(progress_update)?;

        // ファイルの処理完了か、前回の出力から1秒以上経過していれば進捗状況を出力する
        if is_done || prev_output_time.elapsed().as_secs() >= 1 {
            log::info(&progress_summary.log_line()?);
            prev_output_time = Instant::now();
        }
    }
}

/// 進捗更新メッセージを受信する。
fn receive_progress_update(rx: &Receiver<ProgressUpdate>) -> Result<ProgressUpdate, Errors> {
    match rx.recv() {
        Ok(message) => Ok(message),
        Err(error) => Err(log::make_error!("進捗更新メッセージの受信に失敗しました。")
            .with(&error)
            .as_errors()),
    }
}

/// 秒数を時分秒に分割する。
///
/// # Examples
///
/// ```
/// use crate::progress::seconds_to_hms;
/// (h, m, s) = seconds_to_hms(2 * 3600 + 19 * 60 + 37);
/// assert_eq!((h, m, s), (2, 19, 37));
/// ```
fn seconds_to_hms(seconds: u32) -> (u32, u32, u32) {
    (seconds / 3600, seconds / 60 % 60, seconds % 60)
}

/// 進捗サマリー
struct ProgressSummary {
    start_time: Instant,
    disk_progresses: Vec<DiskProgress>,
}

impl ProgressSummary {
    fn new() -> ProgressSummary {
        ProgressSummary {
            disk_progresses: vec![],
            start_time: Instant::now(),
        }
    }

    /// 進捗更新メッセージでこの進捗サマリーを更新する。
    fn update(&mut self, update_info: ProgressUpdate) -> Result<(), Errors> {
        let disk_progress = self.get_disk_progress(update_info.disk_index);
        disk_progress
            .status
            .check_status(&update_info.message_type)?;
        disk_progress.update_by(update_info);

        Ok(())
    }

    /// 指定されたインデックスのディスク進捗を返す。
    /// ディスク進捗が存在しなければ作成する。
    fn get_disk_progress(&mut self, index: usize) -> &mut DiskProgress {
        // ディスク進捗がなければ作成する
        while self.disk_progresses.len() < index + 1 {
            self.disk_progresses.push(DiskProgress::empty_instance());
        }

        self.disk_progresses.get_mut(index).unwrap()
    }

    /// ログに出力する1行の文字列を作成する。
    fn log_line(&self) -> Result<String, Errors> {
        match self.disk_progresses.len() {
            n if n == 1 => Ok(self.log_line_for_single_disk()),
            n if n > 1 => Ok(self.log_line_for_multiple_disks()),
            _ => Err(log::make_error!(
                "ディスク情報が1つもない状態で進捗ログ出力が実行されました。"
            )
            .as_errors()),
        }
    }

    /// ディスク情報が1つである場合のログ出力行を作成する。
    fn log_line_for_single_disk(&self) -> String {
        let disk_progress = &self.disk_progresses[0];

        let mut line = String::new();

        // ディスクID
        // このディスクの初期化メッセージは受信しているので未設定である可能性はない
        line.push_str(disk_progress.disk_id.as_ref().unwrap().as_str());
        line.push(' ');
        // 完了ファイル数/総ファイル数
        write!(line, "{:5}", disk_progress.number_of_done_files).unwrap();
        line.push('/');
        if disk_progress.status == DiskProgressStatus::Initialized {
            line.push_str("-----");
        } else {
            write!(line, "{:5}", disk_progress.number_of_files).unwrap();
        }
        line.push(' ');
        // 進捗率
        if disk_progress.status == DiskProgressStatus::Calculating
            || disk_progress.status == DiskProgressStatus::WaitNewFile
        {
            write!(line, "{:6.2}", disk_progress.rate() * 100.0).unwrap();
        } else {
            line.push_str("  -.--");
        }
        line.push('%');
        line.push(' ');
        // 残り時間
        if disk_progress.red_size > 0 {
            let (hours, minutes, seconds) =
                seconds_to_hms(disk_progress.remain_time_seconds(&self.start_time));
            write!(line, "{:3}:{:02}:{:02}", hours, minutes, seconds).unwrap();
        } else {
            line.push_str("  -:--:--");
        }

        // 処理中ファイル
        if let Some(current_file) = &disk_progress.current_file {
            line.push(' ');
            line.push_str(current_file.to_str().unwrap());
        }

        line
    }

    /// ディスク情報が複数である場合のログ出力行を作成する。
    fn log_line_for_multiple_disks(&self) -> String {
        let mut line = String::new();
        let mut show_remain_time = false;
        let mut max_remain_time_seconds = 0;

        for disk_progress in self.disk_progresses.iter() {
            if disk_progress.status == DiskProgressStatus::New {
                continue;
            }

            if line.len() > 0 {
                line.push_str(" / ");
            }

            // ディスクID
            line.push_str(disk_progress.disk_id.as_ref().unwrap().as_str());
            line.push(' ');
            // 進捗率
            if disk_progress.status == DiskProgressStatus::Initialized {
                write!(line, "{:6.2}", disk_progress.rate()).unwrap();
            } else {
                line.push_str("  -.--");
            }
            line.push('%');

            // 残り時間の最大を更新する
            if disk_progress.red_size > 0 {
                let remain_time_seconds = disk_progress.remain_time_seconds(&self.start_time);
                if remain_time_seconds > max_remain_time_seconds {
                    max_remain_time_seconds = remain_time_seconds;
                }
                show_remain_time = true;
            }
        }

        if show_remain_time {
            line.push_str(" - ");

            let (hours, minutes, seconds) = seconds_to_hms(max_remain_time_seconds);
            write!(line, "{:3}:{:02}:{:02}", hours, minutes, seconds).unwrap();
        }

        line
    }
}

#[derive(Debug, PartialEq)]
enum DiskProgressStatus {
    /// 新規
    New,
    /// 初期化済み
    /// ディスクIDが設定されている。
    Initialized,
    /// 新規ファイル待ち
    /// 初夏期直後、またはファイルの処理が終わり、次のファイルの計算が始まるのを待っている。
    WaitNewFile,
    /// 計算中
    /// ファイルのハッシュ計算を行っている。
    Calculating,
}

impl DiskProgressStatus {
    /// 進捗更新メッセージの種別とディスク進捗のステータスの整合性を確認する。
    fn check_status(&self, message_type: &ProgressUpdateType) -> Result<(), Errors> {
        let ok = match self {
            DiskProgressStatus::Calculating => {
                *message_type == ProgressUpdateType::Read
                    || *message_type == ProgressUpdateType::Done
            }
            DiskProgressStatus::New => *message_type == ProgressUpdateType::Init,
            DiskProgressStatus::Initialized => *message_type == ProgressUpdateType::ListTargets,
            DiskProgressStatus::WaitNewFile => *message_type == ProgressUpdateType::NewFile,
        };

        match ok {
            true => Ok(()),
            false => self.status_errors(message_type),
        }
    }

    /// ステータスエラー情報を作成する。
    fn status_errors(&self, message_type: &ProgressUpdateType) -> Result<(), Errors> {
        Err(log::make_error!(
            "進捗更新メッセージの種別が不正です。: status={:?} message_type={:?}",
            self,
            message_type
        )
        .as_errors())
    }
}

/// ディスク進捗
struct DiskProgress {
    status: DiskProgressStatus,
    disk_id: Option<String>,
    number_of_files: usize,
    number_of_done_files: usize,
    total_size: u64,
    red_size: u64,
    current_file: Option<PathBuf>,
}

impl DiskProgress {
    fn empty_instance() -> DiskProgress {
        DiskProgress {
            status: DiskProgressStatus::New,
            disk_id: None,
            number_of_files: 0,
            number_of_done_files: 0,
            total_size: 0,
            red_size: 0,
            current_file: None,
        }
    }

    /// 指定された進捗更新メッセージでディスク進捗を更新する。
    fn update_by(&mut self, update_info: ProgressUpdate) {
        match update_info.message_type {
            ProgressUpdateType::Init => {
                self.status = DiskProgressStatus::Initialized;
                self.disk_id = update_info.disk_id;
            }
            ProgressUpdateType::ListTargets => {
                self.status = DiskProgressStatus::WaitNewFile;
                self.number_of_files = update_info.number_of_files;
                self.total_size = update_info.total_size;
            }
            ProgressUpdateType::NewFile => {
                self.status = DiskProgressStatus::Calculating;
                self.current_file = update_info.file_path;
            }
            ProgressUpdateType::Read => {
                self.red_size += update_info.red_size;
            }
            ProgressUpdateType::Done => {
                self.status = DiskProgressStatus::WaitNewFile;
                self.number_of_done_files += 1;
            }
        }
    }

    /// 進捗率を計算する。
    fn rate(&self) -> f64 {
        (self.red_size as f64) / (self.total_size as f64)
    }

    /// 残り時間の秒数を計算する。
    fn remain_time_seconds(&self, start_time: &Instant) -> u32 {
        let seconds = start_time.elapsed().as_secs() as f64;
        (seconds / self.rate() - seconds) as u32
    }
}

/// 進捗更新メッセージタイプ
#[derive(Debug, PartialEq)]
enum ProgressUpdateType {
    Init,
    ListTargets,
    NewFile,
    Read,
    Done,
}

/// 進捗更新メッセージ
pub struct ProgressUpdate {
    message_type: ProgressUpdateType,
    disk_index: usize,
    disk_id: Option<String>,
    number_of_files: usize,
    total_size: u64,
    file_path: Option<PathBuf>,
    red_size: u64,
}

const EMPTY_PROGRESS_UPDATE: ProgressUpdate = ProgressUpdate {
    message_type: ProgressUpdateType::Init,
    disk_index: 0,
    disk_id: None,
    number_of_files: 0,
    total_size: 0,
    file_path: None,
    red_size: 0,
};

impl ProgressUpdate {
    pub fn init(disk_id: String) -> ProgressUpdate {
        ProgressUpdate {
            message_type: ProgressUpdateType::Init,
            disk_id: Some(disk_id),
            ..EMPTY_PROGRESS_UPDATE
        }
    }

    pub fn list_targets(number_of_files: usize, total_size: u64) -> ProgressUpdate {
        ProgressUpdate {
            message_type: ProgressUpdateType::ListTargets,
            number_of_files,
            total_size,
            ..EMPTY_PROGRESS_UPDATE
        }
    }

    pub fn new_file(filepath: PathBuf) -> ProgressUpdate {
        ProgressUpdate {
            message_type: ProgressUpdateType::NewFile,
            file_path: Some(filepath),
            ..EMPTY_PROGRESS_UPDATE
        }
    }

    pub fn read(red_size: u64) -> ProgressUpdate {
        ProgressUpdate {
            message_type: ProgressUpdateType::Read,
            red_size,
            ..EMPTY_PROGRESS_UPDATE
        }
    }

    pub fn done() -> ProgressUpdate {
        ProgressUpdate {
            message_type: ProgressUpdateType::Done,
            ..EMPTY_PROGRESS_UPDATE
        }
    }
}
