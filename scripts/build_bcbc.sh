#!/bin/bash
pushd $BCBCDEV
if [ "$GOOS" = "windows" ]; then
  OUTPUT="bcbc.exe"
else
  OUTPUT="bcbc"
fi
go build -o build/$OUTPUT cmd/bcbc/main.go
popd
