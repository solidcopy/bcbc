package main

import (
	"flag"
	"github.com/solidcopy/bcbc/internal/app/bcbc"
)

func main() {
	flag.Parse()
	bcbc.Execute(flag.Args())
}
