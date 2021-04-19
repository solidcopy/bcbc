package calc

import (
	"bufio"
	"io/ioutil"
	"log"
	"os"
	"path"
	"regexp"
	"strings"
)

func Execute() {
	log.Println("ハッシュ計算を開始します。")

	diskFiles := findDiskFiles()
	if len(diskFiles) == 0 {
		log.Printf("diskファイルが見つかりませんでした。")
	}

	queue := make(chan bool)

	diskInfoList := diskRoots(diskFiles)
	for _, di := range diskInfoList {
		go func(di *DiskInfo) {
			defer func() { queue <- true }()

			hashedFileList := hashedFileList(di.hashFile())
			for _, hashedFile := range hashedFileList {
				log.Println(hashedFile)
			}
		}(&di)
	}

	for range diskInfoList {
		<-queue
	}

	log.Println("ハッシュ計算を終了しました。")
}

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
	path  string
	group string
	id    string
}

// diskファイルの一覧からディスク情報のスライスを作成する。
func diskRoots(diskFiles []string) []DiskInfo {
	var diskInfoList = make([]DiskInfo, 0, len(diskFiles))

	pattern := regexp.MustCompile("\\A([A-Z]\\d+)")

	for _, diskFile := range diskFiles {
		diskFileData, err := ioutil.ReadFile(diskFile)
		if err != nil {
			log.Println("diskファイルが読み込めませんでした。:", err)
			continue
		}

		match := pattern.FindStringSubmatch(string(diskFileData))
		if match == nil {
			log.Printf("diskファイルの内容が不正です。: %s", diskFile)
			continue
		}

		id := match[0]
		group := id[0:1]

		diskInfoList = append(diskInfoList, DiskInfo{diskFile, group, id})
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

// ハッシュファイルからハッシュ計算済みのファイル一覧を作成する。
func hashedFileList(hashFile string) []string {

	hashFileIn, err := os.Open(hashFile)
	if err != nil {
		log.Println("ハッシュファイルの読み込みに失敗しました。:", hashFile)
		return []string{}
	}
	defer hashFileIn.Close()

	result := make([]string, 0, 1024)

	hashFileScanner := bufio.NewScanner(hashFileIn)
	for hashFileScanner.Scan() {
		line := hashFileScanner.Text()

		tokens := strings.Split(line, ":")
		if len(tokens) != 2 {
			log.Println("ハッシュファイルが破損しています。:", hashFile)
			return []string{}
		}

		filePath := tokens[0]
		result = append(result, filePath)
	}

	return result
}
