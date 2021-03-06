package bcbc

import (
	"bufio"
	"golang.org/x/text/unicode/norm"
	"io"
	"log"
	"os"
	"path"
	"regexp"
	"strings"
	"time"
)

// Config 設定
type Config struct {
	homeDir string
	filters []Filter
}

// 設定
var config Config

// ログディレクトリを返す。
func (c *Config) logDir() string {
	return path.Join(config.homeDir, "log")
}

// 出力ディレクトリを返す。
func (c *Config) outDir() string {
	return path.Join(config.homeDir, "out")
}

// 設定ディレクトリを返す。
func (c *Config) configDir() string {
	return path.Join(config.homeDir, "configs")
}

// EnvHome 環境変数名: BCBCホームディレクトリ
const EnvHome = "BCBCHOME"

// 環境変数を取得する。
func initEnvs() {
	value, found := os.LookupEnv(EnvHome)
	if !found {
		log.Fatalf("環境変数%sが設定されていません。\n", EnvHome)
	}
	config.homeDir = value
}

// ロガーを初期化する
func initLogger() *os.File {
	err := os.MkdirAll(config.logDir(), 0755)
	fatalMessageError(err, "ログディレクトリを作成できませんでした。: %s\n", config.logDir())

	logFileName := time.Now().Format("20060102150405.log")
	logFilePath := path.Join(config.logDir(), logFileName)
	logFileOut, err := os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY, 0644)
	fatalMessageError(err, "ログファイルを作成できませんでした。: %s\n", logFilePath)

	logf = log.New(io.MultiWriter(os.Stdout, logFileOut), "", log.LstdFlags)

	return logFileOut
}

// Filter フィルター
type Filter struct {
	pattern   *regexp.Regexp
	inclusion bool
}

// フィルター設定を読み込む。
func initFilters() {

	filterConfigFile := path.Join(config.configDir(), "filter.conf")
	filterFileIn, err := os.Open(filterConfigFile)
	fatalMessageError(err, "フィルター設定ファイルが見つかりません。\n")
	defer filterFileIn.Close()

	config.filters = make([]Filter, 0)

	filterFileScanner := bufio.NewScanner(filterFileIn)
	for i := 1; filterFileScanner.Scan(); i++ {
		line := filterFileScanner.Text()
		line = norm.NFC.String(line)

		if strings.TrimSpace(line) == "" || line[0] == '#' {
			continue
		}

		malformed := len(line) < 2 || (line[0] != '+' && line[0] != '-')
		fatalMessageIf(malformed, "フィルター設定ファイルの形式が不正です。: %d行目: %s\n", i, line)

		pattern, err := regexp.Compile(line[1:])
		fatalMessageError(err, "フィルター設定ファイルの形式が不正です。: %d行目: %s\n", i, line)

		inclusion := line[0] == '+'
		filter := Filter{pattern, inclusion}
		config.filters = append(config.filters, filter)
	}
}
