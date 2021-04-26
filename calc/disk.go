package calc

import (
	"io/fs"
	"log"
	"os"
	"path"
	"regexp"
)

// diskファイルを探して一覧を作成する。
func findDiskFiles(diskRoots []string) []string {
	diskFiles := make([]string, 0, 1)

	if len(diskRoots) == 0 {
		diskFile, err := findDiskFileFromCurrent()
		if err != nil {
			diskFiles = append(diskFiles, diskFile)
		}
	} else {
		for _, dr := range diskRoots {
			diskFile := path.Join(dr, "disk")
			_, err := os.Stat(diskFile)
			if err == nil {
				diskFiles = append(diskFiles, diskFile)
			} else {
				log.Println("指定されたディレクトリのdiskファイルが読み込めませんでした。:", dr)
			}
		}
	}

	return diskFiles
}

func findDiskFileFromCurrent() (string, error) {
	dir, err := os.Getwd()
	if err != nil {
		log.Fatalln(err)
	}

	for {
		diskFile := path.Join(dir, "disk")

		_, err := os.Stat(diskFile)
		if err == nil {
			return diskFile, nil
		}

		if dir == "/" {
			break
		}

		dir = path.Dir(dir)
	}

	return "", fs.ErrNotExist
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

	for _, diskFile := range diskFiles {
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

		index := len(diskInfoList)
		diskPath := path.Dir(diskFile)
		id := match[0]
		group := id[0:1]

		diskInfoList = append(diskInfoList, DiskInfo{index, diskPath, group, id})
	}

	return diskInfoList
}

// ハッシュファイルのパスを返す。
func (di *DiskInfo) hashFile() string {
	mergeDir, found := os.LookupEnv("BCBCMERGE")
	if !found {
		log.Fatalln("環境変数BCBCMERGEが設定されていません。")
	}
	return path.Join(mergeDir, di.id)
}
