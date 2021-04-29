#!/bin/bash
pushd $BCBCDEV
build_bcbc.sh
DEST=$GOBIN
if [ -n "$GOBIN" ]; then
  DEST=$GOBIN
else
  if [ -n "$GOPATH" ]; then
    DEST=$GOPATH/bin
  else
    DEST=$HOME/go/bin
  fi
fi
mv $BCBCDEV/build/bcbc $DEST
popd
