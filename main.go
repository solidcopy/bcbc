package main

import (
	"flag"
	"fmt"
)

func main() {
	service := determineService()
	fmt.Println(service)
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
