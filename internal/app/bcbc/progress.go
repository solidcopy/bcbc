package bcbc

import (
	"fmt"
	"log"
	"strings"
	"time"
)

// ProgressInfo 進捗情報
type ProgressInfo struct {
	diskInfo       *DiskInfo
	sizeCount      ProgressCount
	fileCount      ProgressCount
	processingFile string
	startTime      time.Time
}

type ProgressCount struct {
	total     uint64
	processed uint64
}

func (pc *ProgressCount) ProgressRate() float64 {
	if pc.total == 0 {
		return 1.0
	}
	return float64(pc.processed) / float64(pc.total)
}

func (pc *ProgressCount) Completed() bool {
	return pc.processed >= pc.total
}

func (pc *ProgressCount) Increment(n uint64) {
	pc.processed += n
}

// 進捗監視ルーチン。
func watchProgress(numberOfDisks int, progressChannel chan ProgressInfo) {
	progressInfoList := make([]ProgressInfo, numberOfDisks)

	lastPrintTime := time.Now()

	for {
		progressInfo := <-progressChannel
		progressInfoList[progressInfo.diskInfo.index] = progressInfo

		if time.Now().Sub(lastPrintTime) >= time.Second {
			if numberOfDisks == 1 {
				printProgress(progressInfoList[0])
			} else {
				printProgressSummary(progressInfoList)
			}
			lastPrintTime = time.Now()
		}
	}
}

// 1つのディスク処理について進捗情報を表示する。
func printProgress(progressInfo ProgressInfo) {

	if progressInfo.diskInfo == nil {
		return
	}

	fc := progressInfo.fileCount
	sc := progressInfo.sizeCount
	rate := sc.ProgressRate()
	remainTime := calcRemainTime(progressInfo.startTime, rate)
	formattedRemainTime := formatRemainTime(remainTime)

	log.Printf("%s [%5d/%5d] %6.2f%% %s %s\n",
		progressInfo.diskInfo.id, fc.processed, fc.total, rate*100, formattedRemainTime, progressInfo.processingFile)
}

// 複数のディスク処理について進捗情報の概要を表示する。
func printProgressSummary(progressInfoList []ProgressInfo) {
	summaries := make([]string, 0, len(progressInfoList))

	maxRemainTime := int64(0)

	for _, pi := range progressInfoList {
		if pi.diskInfo != nil {
			rate := pi.sizeCount.ProgressRate()
			summaries = append(summaries, fmt.Sprintf("%s %6.2f%%", pi.diskInfo.id, rate*100))

			remainTime := calcRemainTime(pi.startTime, rate)
			if remainTime > maxRemainTime {
				maxRemainTime = remainTime
			}
		}
	}

	log.Println(strings.Join(summaries, " / "), "-", formatRemainTime(maxRemainTime))
}

// 残り時間を計算してhhh:mm:ss形式の文字列にフォーマットする。
func formatRemainTime(remainTime int64) string {
	if remainTime == -1 {
		return "  -:--:--"
	}

	hours := remainTime / int64(time.Hour)
	remainTime -= hours * int64(time.Hour)
	minutes := remainTime / int64(time.Minute)
	remainTime -= minutes * int64(time.Minute)
	seconds := remainTime / int64(time.Second)

	return fmt.Sprintf("%3d:%02d:%02d", hours, minutes, seconds)
}

// 残り時間を計算する。
func calcRemainTime(startTime time.Time, rate float64) int64 {
	if rate == 0 {
		return -1
	}
	elapsedTime := float64(time.Now().Sub(startTime))
	return int64(elapsedTime/rate - elapsedTime)
}
