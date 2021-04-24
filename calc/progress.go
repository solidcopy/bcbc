package calc

import (
	"log"
	"time"
)

// ファイル総数を計算する。
func calcTotalFiles(fileInfoList []FileInfo) uint {
	var totalFiles int
	for _, fi := range fileInfoList {
		if fi.StatSuccess() {
			totalFiles++
		}
	}
	return uint(totalFiles)
}

// ファイルサイズの合計を計算する。
func calcTotalSize(fileInfoList []FileInfo) uint64 {
	var totalSize uint64
	for _, fi := range fileInfoList {
		totalSize += fi.Size()
	}
	return totalSize
}

// ProgressInfo 進捗情報
type ProgressInfo struct {
	diskInfo       *DiskInfo
	sizeCount      ProgressCount
	fileCount      ProgressCount
	processingFile string
}

type ProgressCount struct {
	total     uint64
	processed uint64
}

func (pc *ProgressCount) ProgressRate() float64 {
	return float64(pc.processed) / float64(pc.total)
}

func (pc *ProgressCount) Completed() bool {
	return pc.processed >= pc.total
}

func (pc *ProgressCount) Increment(n uint64) {
	pc.processed += n
}

// 進捗監視ルーチン。
func watchProgress(diskInfoList []DiskInfo, progressChannel chan ProgressInfo, quitChannel chan bool) {
	progressInfoList := make([]ProgressInfo, len(diskInfoList))

	lastPrintTime := time.Now()

	for {
		progressInfo := <-progressChannel
		progressInfoList[progressInfo.diskInfo.index] = progressInfo

		allCompleted := true
		for _, pi := range progressInfoList {
			if !pi.fileCount.Completed() {
				allCompleted = false
				break
			}
		}
		if allCompleted {
			printProgress(progressInfoList)
			break
		}

		now := time.Now()
		if now.Sub(lastPrintTime) >= time.Second {
			printProgress(progressInfoList)
			lastPrintTime = now
		}
	}

	quitChannel <- true
}

// 進捗情報を表示する。
func printProgress(progressInfoList []ProgressInfo) {

	for _, pi := range progressInfoList {
		fc := pi.fileCount
		sc := pi.sizeCount
		log.Printf("%s [%5d/%5d] %3.2f%% %s\n", pi.diskInfo.id, fc.processed, fc.total, sc.ProgressRate()*100, pi.processingFile)
	}
}
