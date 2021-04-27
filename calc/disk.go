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
	var diskFiles []string

	if len(diskRoots) == 0 {
		diskFile, err := findDiskFileFromCurrent()
		if err != nil {
			log.Fatalln("diskファイルが見つかりませんでした。")
		}
		diskFiles = []string{diskFile}
	} else {
		diskFiles = make([]string, 0, len(diskRoots))
		for _, dr := range diskRoots {
			diskFile := path.Join(dr, "disk")
			diskFiles = append(diskFiles, diskFile)
		}
	}

	return diskFiles
}

// カレントディレクトリの起点としてdiskファイルを探す。
func findDiskFileFromCurrent() (string, error) {
	dir, err := os.Getwd()
	if err != nil {
		log.Fatalln(err)
	}

	for {
		diskFile := path.Join(dir, "disk")

		if _, err := os.Stat(diskFile); err == nil {
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
	index    int
	id       string
	rootPath string
}

// diskファイルの一覧からディスク情報のスライスを作成する。
func makeDiskInfoList(diskFiles []string) []DiskInfo {
	diskInfoList := make([]DiskInfo, 0, len(diskFiles))

	pattern := regexp.MustCompile("\\A([A-Z]\\d+)")

	for _, diskFile := range diskFiles {
		diskFileData, err := os.ReadFile(diskFile)
		if err != nil {
			log.Fatalln("diskファイルが読み込めませんでした。:", err)
		}

		match := pattern.FindStringSubmatch(string(diskFileData))
		if match == nil {
			log.Fatalln("diskファイルの内容が不正です。:", diskFile)
		}

		index := len(diskInfoList)
		id := match[0]
		rootPath := path.Dir(diskFile)

		diskInfoList = append(diskInfoList, DiskInfo{index, id, rootPath})
	}

	return diskInfoList
}

// hashFile ハッシュファイルのパスを返す。
func (di *DiskInfo) hashFile() string {
	return path.Join(config.envs[EnvMerge], di.id)
}
