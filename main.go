package main

import (
	"flag"
	"log"
)

import (
	"github.com/video_backup_checker/calc"
	"github.com/video_backup_checker/merge"
)

func main() {
	service := determineService()
	runService(service)
}

// コマンドライン引数から実行するサービスを決定する。
func determineService() (service string) {
	flag.Parse()

	service = flag.Arg(0)
	if service == "" {
		service = "calc"
	}

	return
}

// 指定されたサービスを実行する。
func runService(service string) {
	switch service {
	case "calc":
		calc.Execute()
	case "merge":
		merge.Execute()
	default:
		log.Fatalf("不正なサービス指定: %v", service)
	}
}
