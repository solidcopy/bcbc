package bcbc

import (
	"bufio"
	"golang.org/x/text/unicode/norm"
	"io/fs"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"
)

// FileInfo ファイル情報
type FileInfo struct {
	diskInfo *DiskInfo
	realPath string
	normPath string
	_size    int64
}

// ファイル情報を初期化する。
func (fi *FileInfo) init(diskInfo *DiskInfo, realPath string) {
	fi.diskInfo = diskInfo

	fi.realPath = realPath

	normPath, _ := filepath.Rel(diskInfo.rootPath, realPath)
	normPath = filepath.ToSlash(normPath)
	normPath = norm.NFC.String(normPath)
	fi.normPath = normPath

	// sizeメソッドで遅延初期化する
	fi._size = -1
}

func (fi *FileInfo) size() (uint64, error) {
	if fi._size != -1 {
		return uint64(fi._size), nil
	}

	stat, err := os.Stat(fi.realPath)
	if err == nil {
		fi._size = stat.Size()
		return uint64(fi._size), nil
	} else {
		return 0, nil
	}
}

// ハッシュ対象ファイルの一覧を作成する。
func listFileInfo(diskInfo *DiskInfo) ([]FileInfo, uint64) {

	hashMap := makeHashMap(diskInfo)

	trimmedHashs := strings.Builder{}

	files := listFiles(diskInfo.rootPath)

	capacity := len(files) - len(hashMap)
	if capacity < 0 {
		capacity = 0
	}

	fileInfoList := make([]FileInfo, 0, capacity)

	var totalSize uint64

	var fileInfo FileInfo
	for _, file := range files {

		(&fileInfo).init(diskInfo, file)

		hash, found := hashMap[fileInfo.normPath]
		if found {
			_, err := trimmedHashs.WriteString(fileInfo.normPath + ":" + hash + "\n")
			fatalMessageError(err, "ハッシュファイルの書き込みに失敗しました。\n")
			continue
		}

		if filterFile(fileInfo.normPath) {
			fileInfoList = append(fileInfoList, fileInfo)
			size, err := fileInfo.size()
			fatalMessageError(err, "ファイルサイズの取得に失敗しました。: %s\n", fileInfo.realPath)
			totalSize += size
		}
	}

	err := ioutil.WriteFile(diskInfo.hashFile(), []byte(trimmedHashs.String()), 0644)
	fatalMessageError(err, "ハッシュファイルの作成に失敗しました。\n")

	return fileInfoList, totalSize
}

// ハッシュファイルからハッシュ計算済みのファイルセットを作成する。
func makeHashMap(diskInfo *DiskInfo) map[string]string {

	hashFileIn, err := os.Open(diskInfo.hashFile())
	if err != nil {
		return map[string]string{}
	}
	defer hashFileIn.Close()

	result := make(map[string]string, 1024)

	hashFileScanner := bufio.NewScanner(hashFileIn)
	for i := 1; hashFileScanner.Scan(); i++ {
		line := hashFileScanner.Text()

		tokens := strings.Split(line, ":")
		fatalMessageIf(len(tokens) != 2, "ハッシュファイルが破損しています。: %s : %d行目:\n", diskInfo.hashFile(), i)

		result[tokens[0]] = tokens[1]
	}

	return result
}

// ディスク内のファイル一覧を作成する。
func listFiles(rootPath string) []string {

	result := make([]string, 0)

	err := filepath.WalkDir(rootPath, func(path string, dirEntry fs.DirEntry, err error) error {
		if !dirEntry.IsDir() {
			result = append(result, path)
		}
		return nil
	})
	fatalMessageError(err, "ファイル一覧の作成中にエラーが発生しました。\n")

	return result
}

// 指定されたファイルがハッシュ対象であるかフィルター設定から判定する。
func filterFile(normPath string) bool {
	for _, filter := range config.filters {
		if filter.pattern.MatchString(normPath) {
			return filter.inclusion
		}
	}

	return false
}
