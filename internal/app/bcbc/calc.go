package bcbc

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"sort"
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

	logf.Println("ハッシュファイルの統合を開始します。")

	mergedHashMap := make(map[string][]string)

	outputFiles, err := filepath.Glob(path.Join(config.outDir(), "*"))
	if err != nil {
		logf.Println("出力ファイルの一覧取得に失敗しました。")
		logf.Fatal(err)
	}

	hashFilePattern := regexp.MustCompile("^([A-Z])\\d+$")

	for _, outputFile := range outputFiles {
		fileName := filepath.Base(outputFile)
		subMatches := hashFilePattern.FindStringSubmatch(fileName)
		if len(subMatches) == 0 {
			continue
		}

		group := subMatches[1]

		mergedHashes := mergedHashMap[group]

		hashFileIn, err := os.Open(outputFile)
		if err != nil {
			logf.Println("ハッシュファイルの読み込みに失敗しました。", outputFile)
			logf.Fatalln(err)
		}
		for hashFileScanner := bufio.NewScanner(hashFileIn); hashFileScanner.Scan(); {
			line := hashFileScanner.Text()
			if line != "" {
				mergedHashes = append(mergedHashes, line)
			}
		}
		mergedHashMap[group] = mergedHashes
	}

	for group, mergedHashs := range mergedHashMap {
		sort.Strings(mergedHashs)
		mergedHashFile := path.Join(config.outDir(), group)

		mergedHashFileOut, err := os.OpenFile(mergedHashFile, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0644)
		if err != nil {
			logf.Println("統合ハッシュファイルの作成に失敗しました。")
			logf.Fatal(err)
		}

		for _, line := range mergedHashs {
			_, err := fmt.Fprintln(mergedHashFileOut, line)
			if err != nil {
				logf.Println("統合ハッシュファイルの書き込みに失敗しました。")
				logf.Fatal(err)
			}
		}

		mergedHashFileOut.Close()
	}

	logf.Println("ハッシュファイルの統合を終了しました。")
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
