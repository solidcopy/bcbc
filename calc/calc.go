package calc

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"
)

func Execute(diskRoots []string) {
	log.Println("ハッシュ計算を開始します。")

	diskFiles := findDiskFiles(diskRoots)
	if len(diskFiles) == 0 {
		log.Fatalln("diskファイルが見つかりませんでした。")
	}

	progressChannel := make(chan ProgressInfo)
	quitChannel := make(chan bool)
	diskInfoList := makeDiskInfoList(diskFiles)
	go watchProgress(len(diskInfoList), progressChannel)

	for i := range diskInfoList {
		go hashRoutine(diskInfoList, i, progressChannel, quitChannel)
	}

	// 全ハッシュルーチンの終了を待つ
	for range diskInfoList {
		<-quitChannel
	}

	log.Println("ハッシュ計算を終了しました。")
}

// ハッシュルーチン。
func hashRoutine(diskInfoList []DiskInfo, i int, progressChannel chan ProgressInfo, quitChannel chan bool) {
	di := &diskInfoList[i]

	hashFileIn, err := os.OpenFile(di.hashFile(), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		log.Println(err)
		quitChannel <- false
		return
	}
	defer hashFileIn.Close()

	targetFiles := listTargetFiles(di)
	fileInfoList := toFileInfoList(targetFiles)

	totalFiles := calcTotalFiles(fileInfoList)
	totalSize := calcTotalSize(fileInfoList)
	progressInfo := ProgressInfo{
		diskInfo:  di,
		fileCount: ProgressCount{uint64(totalFiles), 0},
		sizeCount: ProgressCount{totalSize, 0},
		startTime: time.Now(),
	}
	progressChannel <- progressInfo

	failFiles := make([]string, 0)

	for _, fi := range fileInfoList {
		if !fi.StatSuccess() {
			failFiles = append(failFiles, fi.path)
			continue
		}

		hash, err := calcHash(fi.path, progressInfo, progressChannel)

		progressInfo.fileCount.Increment(uint64(1))
		progressInfo.sizeCount.Increment(fi.Size())

		if err != nil {
			failFiles = append(failFiles, fi.path)
			continue
		}

		relativePath, _ := filepath.Rel(di.path, fi.path)
		relativePath = filepath.ToSlash(relativePath)
		_, err = fmt.Fprintf(hashFileIn, "%s:%x\n", relativePath, hash)
		if err == nil {
			err = hashFileIn.Sync()
		}
		if err != nil {
			log.Println(err)
			quitChannel <- false
			return
		}
	}

	progressChannel <- progressInfo

	quitChannel <- true
}
