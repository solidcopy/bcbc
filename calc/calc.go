package calc

import (
	"fmt"
	"log"
	"os"
	"time"
)

// ロガー。
// 標準出力とログファイルにログを出力する。
var logf *log.Logger

func Execute(diskRoots []string) {

	// 初期処理
	initEnvs()
	logFileOut := initLogger()
	defer logFileOut.Close()
	initFilters()

	logf.Println("ハッシュ計算を開始します。")

	diskFiles := findDiskFiles(diskRoots)
	if len(diskFiles) == 0 {
		logf.Fatalln("diskファイルが見つかりませんでした。")
	}

	progressChannel := make(chan ProgressInfo)
	completionChannel := make(chan CompletionMessage)
	diskInfoList := makeDiskInfoList(diskFiles)
	go watchProgress(len(diskInfoList), progressChannel)

	for i := range diskInfoList {
		go hashRoutine(&diskInfoList[i], progressChannel, completionChannel)
	}

	// 全ハッシュルーチンの終了を待つ
	for range diskInfoList {
		if completion := <-completionChannel; completion.err != nil {
			logf.Printf("ディスク(%s)のハッシュ計算中に問題が発生しました。\n", completion.diskId)
			logf.Println(completion.err)
		}
	}

	logf.Println("ハッシュ計算を終了しました。")
}

// CompletionMessage 完了メッセージ
type CompletionMessage struct {
	diskId string
	err    error
}

// ハッシュルーチン。
func hashRoutine(diskInfo *DiskInfo, progressChannel chan ProgressInfo, completionChannel chan CompletionMessage) {

	err := os.MkdirAll(config.outDir(), 0755)
	if err != nil {
		log.Println("出力ディレクトリを作成できませんでした。")
		log.Fatalln(err)
	}

	hashFileOut, err := os.OpenFile(diskInfo.hashFile(), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		logf.Fatalln("ハッシュファイルの書き込みに失敗しました。", err)
	}
	defer hashFileOut.Close()

	fileInfoList, totalSize := listFileInfo(diskInfo)

	progressInfo := ProgressInfo{
		diskInfo:  diskInfo,
		fileCount: ProgressCount{uint64(len(fileInfoList)), 0},
		sizeCount: ProgressCount{totalSize, 0},
		startTime: time.Now(),
	}
	progressChannel <- progressInfo

	for _, fi := range fileInfoList {

		hash, err := calcHash(fi.realPath, progressInfo, progressChannel)

		progressInfo.fileCount.Increment(uint64(1))
		size, _ := fi.size()
		progressInfo.sizeCount.Increment(size)

		if err != nil {
			logf.Println("ハッシュ計算中にエラーが発生しました。")
			logf.Println(fi.realPath)
			logf.Println(err)
			continue
		}

		_, err = fmt.Fprintf(hashFileOut, "%s:%x\n", fi.normPath, hash)
		if err == nil {
			err = hashFileOut.Sync()
		}
		if err != nil {
			completionChannel <- CompletionMessage{diskInfo.id, err}
			return
		}
	}

	progressChannel <- progressInfo

	completionChannel <- CompletionMessage{diskInfo.id, nil}
}
