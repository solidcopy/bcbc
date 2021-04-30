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
)

// ロガー。
// 標準出力とログファイルにログを出力する。
var logf *log.Logger

// Execute エントリーポイント。
func Execute(diskRoots []string) {

	// 初期処理
	initEnvs()
	logFileOut := initLogger()
	defer logFileOut.Close()
	initFilters()

	executeHashCalculation(diskRoots)
	executeHashFileIntegration()
}

// ハッシュ計算を実行する。
func executeHashCalculation(diskRoots []string) {

	logf.Println("ハッシュ計算を開始します。")
	defer logf.Println("ハッシュ計算を終了しました。")

	diskFiles := findDiskFiles(diskRoots)
	fatalMessageIf(len(diskFiles) == 0, "diskファイルが見つかりませんでした。\n")

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
}

// ハッシュファイル統合を実行する。
func executeHashFileIntegration() {

	logf.Println("ハッシュファイルの統合を開始します。")
	defer logf.Println("ハッシュファイルの統合を終了しました。")

	mergedHashMap := make(map[string][]string)

	outputFiles, err := filepath.Glob(path.Join(config.outDir(), "*"))
	fatalMessageError(err, "出力ファイルの一覧取得に失敗しました。\n")

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
		fatalMessageError(err, "ハッシュファイルの読み込みに失敗しました。: %s\n", outputFile)
		for hashFileScanner := bufio.NewScanner(hashFileIn); hashFileScanner.Scan(); {
			line := hashFileScanner.Text()
			if line != "" {
				mergedHashes = append(mergedHashes, line)
			}
		}
		mergedHashMap[group] = mergedHashes
	}

	for group, mergedHashes := range mergedHashMap {
		sort.Strings(mergedHashes)
		mergedHashFile := path.Join(config.outDir(), group)

		mergedHashFileOut, err := os.OpenFile(mergedHashFile, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0644)
		fatalMessageError(err, "統合ハッシュファイルの作成に失敗しました。\n")

		for _, line := range mergedHashes {
			_, err := fmt.Fprintln(mergedHashFileOut, line)
			fatalMessageError(err, "統合ハッシュファイルの書き込みに失敗しました。\n")
		}

		mergedHashFileOut.Close()
	}
}
