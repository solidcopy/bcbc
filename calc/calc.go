package calc

import (
	"log"
)

func Execute() {
	log.Println("ハッシュ計算を開始します。")

	diskFiles := findDiskFiles()
	if len(diskFiles) == 0 {
		log.Fatalln("diskファイルが見つかりませんでした。")
	}

	progressChannel := make(chan ProgressInfo)
	quitChannel := make(chan bool)
	diskInfoList := makeDiskInfoList(diskFiles)
	go watchProgress(diskInfoList, progressChannel, quitChannel)

	for _, di := range diskInfoList {

		go func(di *DiskInfo) {
			targetFiles := listTargetFiles(di)
			fileInfoList := toFileInfoList(targetFiles)

			totalFiles := calcTotalFiles(fileInfoList)
			totalSize := calcTotalSize(fileInfoList)
			progressInfo := ProgressInfo{
				diskInfo:  di,
				fileCount: ProgressCount{uint64(totalFiles), 0},
				sizeCount: ProgressCount{totalSize, 0}}
			progressChannel <- progressInfo

			failFiles := make([]string, 0)

			for _, fi := range fileInfoList {
				if !fi.StatSuccess() {
					failFiles = append(failFiles, fi.path)
					continue
				}

				_, err := calcHash(fi.path, progressInfo, progressChannel)
				progressInfo.fileCount.Increment(uint64(1))
				progressInfo.sizeCount.Increment(fi.Size())

				if err != nil {
					failFiles = append(failFiles, fi.path)
					continue
				}
			}

			progressChannel <- progressInfo

			quitChannel <- true
		}(&di)
	}

	// 全ハッシュルーチンの終了を待つ
	for range diskInfoList {
		<-quitChannel
	}
	// 進捗監視ルーチンの終了を待つ
	<-quitChannel

	log.Println("ハッシュ計算を終了しました。")
}
