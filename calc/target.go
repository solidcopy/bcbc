package calc

import (
	"bufio"
	"io/fs"
	"log"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"strings"
)

// ハッシュ対象ファイルの一覧を作成する。
func listTargetFiles(di *DiskInfo) []string {

	files := listFiles(di.path)
	hashedFileSet := hashedFileSet(di)
	unhashedTargetFileList := removeHashedFiles(files, hashedFileSet)
	filteredTargetFileList := filterFiles(unhashedTargetFileList)

	return filteredTargetFileList
}

// ディスク内のファイル一覧を作成する。
func listFiles(root string) []string {

	result := make([]string, 0, 1024)

	err := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if !d.IsDir() {
			result = append(result, path)
		}
		return nil
	})
	if err != nil {
		log.Fatalln(err)
	}

	return result
}

// ハッシュファイルからハッシュ計算済みのファイルセットを作成する。
func hashedFileSet(di *DiskInfo) map[string]bool {

	hashFile := di.hashFile()
	hashFileIn, err := os.Open(hashFile)
	if err != nil {
		return map[string]bool{}
	}
	defer hashFileIn.Close()

	result := make(map[string]bool, 1024)

	hashFileScanner := bufio.NewScanner(hashFileIn)
	for hashFileScanner.Scan() {
		line := hashFileScanner.Text()

		tokens := strings.Split(line, ":")
		if len(tokens) != 2 {
			log.Println("ハッシュファイルが破損しています。:", hashFile)
			return map[string]bool{}
		}

		filePath := path.Join(di.path, tokens[0])
		result[filePath] = true
	}

	return result
}

// ファイル一覧からハッシュ済みのファイルを除外する。
func removeHashedFiles(fileList []string, hashedFileSet map[string]bool) []string {
	unhashedFileList := make([]string, 0)
	for _, file := range fileList {
		if !hashedFileSet[file] {
			unhashedFileList = append(unhashedFileList, file)
		}
	}
	return unhashedFileList
}

// Filter フィルター
type Filter struct {
	pattern   *regexp.Regexp
	inclusion bool
}

// フィルター一覧
var filters []Filter

// フィルター設定を読み込む。
func init() {
	bcbcHome, found := os.LookupEnv("BCBCHOME")
	if !found {
		log.Fatalln("環境変数BCBCHOMEが設定されていません。")
	}

	filterFile := path.Join(bcbcHome, "config", "filter.conf")
	filterFileIn, err := os.Open(filterFile)
	if err != nil {
		log.Fatalln("フィルター設定ファイルが見つかりません。")
	}
	defer filterFileIn.Close()

	filters = make([]Filter, 0, 10)

	validFilterFile := true

	filterFileScanner := bufio.NewScanner(filterFileIn)
	for filterFileScanner.Scan() {
		line := filterFileScanner.Text()

		if len(line) < 2 || (line[0] != '+' && line[0] != '-') {
			log.Println("1", line)
			validFilterFile = false
			break
		}

		pattern, err := regexp.Compile(line[1:])
		if err != nil {
			log.Println("2", line)
			validFilterFile = false
			break
		}

		inclusion := line[0] == '+'
		filter := Filter{pattern, inclusion}
		filters = append(filters, filter)
	}

	if !validFilterFile {
		log.Fatalln("フィルター設定ファイルの形式が不正です。")
	}
}

// 指定されたファイルがハッシュ対象であるかフィルター設定から判定する。
func filterFile(path string) bool {
	for _, filter := range filters {
		if filter.pattern.MatchString(path) {
			return filter.inclusion
		}
	}

	return false
}

// ファイルをフィルター設定で絞り込む。
func filterFiles(fileList []string) []string {
	filteredFileList := make([]string, 0)
	for _, file := range fileList {
		if filterFile(file) {
			filteredFileList = append(filteredFileList, file)
		}
	}
	return filteredFileList
}

// FileInfo ファイル情報
type FileInfo struct {
	path string
	size int64
}

// Size ファイルサイズを返す。
func (tf *FileInfo) Size() uint64 {
	if tf.StatSuccess() {
		return uint64(tf.size)
	} else {
		return 0
	}
}

// StatSuccess ファイルサイズの取得が成功したかを返す。
func (tf *FileInfo) StatSuccess() bool {
	return tf.size >= 0
}

// ファイル情報リストを作成する。
func toFileInfoList(targetFiles []string) []FileInfo {

	fileInfoList := make([]FileInfo, 0, len(targetFiles))
	for _, tf := range targetFiles {
		stat, err := os.Stat(tf)
		size := int64(-1)
		if err == nil {
			size = stat.Size()
		} else {
			log.Println(err)
		}
		fileInfoList = append(fileInfoList, FileInfo{tf, size})
	}

	return fileInfoList
}
