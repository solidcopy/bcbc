package calc

import (
	"crypto/md5"
	"io"
	"log"
	"os"
)

// BufferSize ファイル読み込み時のバッファサイズ。
const BufferSize = 10 << 20

// ファイルのハッシュを計算する。
func calcHash(file string, progressInfo ProgressInfo, progressInfoChannel chan ProgressInfo) ([]byte, error) {
	fileIn, err := os.Open(file)
	if err != nil {
		log.Println("ハッシュ対象ファイルの読み込みに失敗しました。:", file)
		return nil, err
	}
	defer fileIn.Close()

	progressInfo.processingFile = file

	buffer := make([]byte, BufferSize)

	hasher := md5.New()

	for {
		ret, err := fileIn.Read(buffer)
		if ret == 0 {
			break
		}
		if err != nil && err != io.EOF {
			log.Println(err)
			return nil, err
		}

		hasher.Write(buffer[:ret])

		progressInfo.sizeCount.Increment(uint64(ret))

		progressInfoChannel <- progressInfo
	}

	return hasher.Sum(nil), nil
}
