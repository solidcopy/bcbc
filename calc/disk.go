package calc

import (
	"log"
	"os"
	"path"
	"regexp"
)

// diskファイルを探して一覧を作成する。
func findDiskFiles() []string {
	dir, err := os.Getwd()
	if err != nil {
		log.Fatalln(err)
	}

	for {
		dirEntries, err := os.ReadDir(dir)
		if err != nil {
			log.Fatalln(err)
		}

		for _, dirEntry := range dirEntries {
			if !dirEntry.IsDir() && dirEntry.Name() == "disk" {
				return []string{path.Join(dir, dirEntry.Name())}
			}
		}

		if dir == "/" {
			return []string{}
		} else {
			dir = path.Dir(dir)
		}
	}
}

// DiskInfo ディスク情報
type DiskInfo struct {
	index int
	path  string
	group string
	id    string
}

// diskファイルの一覧からディスク情報のスライスを作成する。
func makeDiskInfoList(diskFiles []string) []DiskInfo {
	var diskInfoList = make([]DiskInfo, 0, len(diskFiles))

	pattern := regexp.MustCompile("\\A([A-Z]\\d+)")

	for i, diskFile := range diskFiles {
		diskFileData, err := os.ReadFile(diskFile)
		if err != nil {
			log.Println("diskファイルが読み込めませんでした。:", err)
			continue
		}

		match := pattern.FindStringSubmatch(string(diskFileData))
		if match == nil {
			log.Println("diskファイルの内容が不正です。:", diskFile)
			continue
		}

		diskPath := path.Dir(diskFile)
		id := match[0]
		group := id[0:1]

		diskInfoList = append(diskInfoList, DiskInfo{i, diskPath, group, id})
	}

	return diskInfoList
}

// ハッシュファイルのパスを返す。
func (di *DiskInfo) hashFile() string {
	mergeDir, found := os.LookupEnv("VBCMERGE")
	if !found {
		log.Fatalln("環境変数VBCMERGEが設定されていません。")
	}
	return path.Join(mergeDir, di.id)
}
