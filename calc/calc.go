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

	queue := make(chan bool)

	diskInfoList := diskRoots(diskFiles)
	for _, di := range diskInfoList {
		go func(di *DiskInfo) {
			defer func() { queue <- true }()

			targetFiles := listTargetFiles(di)

			for _, tf := range targetFiles {
				hash, _ := calcHash(tf)
				log.Printf("%s:%x", tf, hash)
			}
		}(&di)
	}

	for range diskInfoList {
		<-queue
	}

	log.Println("ハッシュ計算を終了しました。")
}
