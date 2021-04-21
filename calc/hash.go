package calc

import (
	"crypto/md5"
	"log"
	"os"
)

// BufferSize ファイル読み込み時のバッファサイズ。
const BufferSize = 2 ^ 20

// ファイルのハッシュを計算する。
func calcHash(file string) ([]byte, error) {
	fileIn, err := os.Open(file)
	if err != nil {
		log.Println("ハッシュ対象ファイルの読み込みに失敗しました。:", file)
		return nil, err
	}
	defer fileIn.Close()

	buffer := make([]byte, BufferSize)

	hasher := md5.New()

	for {
		ret, err := fileIn.Read(buffer)
		if ret == 0 {
			break
		}
		if err != nil {
			log.Println(err)
			return nil, err
		}

		hasher.Write(buffer[:ret])
	}

	return hasher.Sum(nil), nil
}
