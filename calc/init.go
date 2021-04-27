package calc

import (
	"bufio"
	"golang.org/x/text/unicode/norm"
	"log"
	"os"
	"path"
	"regexp"
)

// 初期化処理を行う。
func init() {
	initEnvs()
	initFilters()
}

// Config 設定
var config struct {
	envs    map[string]string
	filters []Filter
}

const EnvHome = "BCBCHOME"
const EnvMerge = "BCBCMERGE"

// 環境変数を取得する。
func initEnvs() {
	config.envs = make(map[string]string)

	value, found := os.LookupEnv(EnvHome)
	if !found {
		log.Fatalf("環境変数%sが設定されていません。\n", EnvHome)
	}
	config.envs[EnvHome] = value

	value, found = os.LookupEnv(EnvMerge)
	if !found {
		value = path.Join(config.envs[EnvHome], "merge")
	}
	config.envs[EnvMerge] = value
}

// Filter フィルター
type Filter struct {
	pattern   *regexp.Regexp
	inclusion bool
}

// フィルター設定を読み込む。
func initFilters() {

	filterConfigFile := path.Join(config.envs[EnvHome], "config", "filter.conf")
	filterFileIn, err := os.Open(filterConfigFile)
	if err != nil {
		log.Fatalln("フィルター設定ファイルが見つかりません。")
	}
	defer filterFileIn.Close()

	config.filters = make([]Filter, 0)

	filterFileScanner := bufio.NewScanner(filterFileIn)
	for filterFileScanner.Scan() {
		line := filterFileScanner.Text()
		line = norm.NFC.String(line)

		if len(line) < 2 || (line[0] != '+' && line[0] != '-') {
			log.Fatalln("フィルター設定ファイルの形式が不正です。")
		}

		pattern, err := regexp.Compile(line[1:])
		if err != nil {
			log.Fatalln(line)
		}

		inclusion := line[0] == '+'
		filter := Filter{pattern, inclusion}
		config.filters = append(config.filters, filter)
	}
}
