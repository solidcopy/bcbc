package bcbc

import (
	"crypto/md5"
	"fmt"
	"io"
	"os"
	"time"
)

// CompletionMessage 完了メッセージ
type CompletionMessage struct {
	diskId string
	err    error
}

// ハッシュルーチン。
func hashRoutine(diskInfo *DiskInfo, progressChannel chan ProgressInfo, completionChannel chan CompletionMessage) {

	err := os.MkdirAll(config.outDir(), 0755)
	fatalMessageError(err, "出力ディレクトリを作成できませんでした。: %s\n", config.outDir())

	hashFileOut, err := os.OpenFile(diskInfo.hashFile(), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	fatalMessageError(err, "ハッシュファイルの書き込みに失敗しました。: %s\n", diskInfo.hashFile())
	defer hashFileOut.Close()

	fileInfoList, totalSize := listFileInfo(diskInfo)

	progressInfo := ProgressInfo{
		diskInfo:  diskInfo,
		fileCount: ProgressCount{uint64(len(fileInfoList)), 0},
		sizeCount: ProgressCount{totalSize, 0},
		startTime: time.Now(),
	}
	progressChannel <- progressInfo

	for _, fi := range fileInfoList {

		hash, err := calcHash(fi.realPath, progressInfo, progressChannel)

		progressInfo.fileCount.Increment(uint64(1))
		size, _ := fi.size()
		progressInfo.sizeCount.Increment(size)

		if err != nil {
			logf.Printf("ハッシュ計算中にエラーが発生しました。: %s\n", fi.realPath)
			logf.Println(err)
			continue
		}

		_, err = fmt.Fprintf(hashFileOut, "%s:%x\n", fi.normPath, hash)
		if err != nil {
			completionChannel <- CompletionMessage{diskInfo.id, err}
			return
		}
	}

	progressChannel <- progressInfo

	completionChannel <- CompletionMessage{diskInfo.id, nil}
}

// BufferSize ファイル読み込み時のバッファサイズ。
const BufferSize = 10 << 20

// ファイルのハッシュを計算する。
func calcHash(file string, progressInfo ProgressInfo, progressInfoChannel chan ProgressInfo) ([]byte, error) {
	fileIn, err := os.Open(file)
	if err != nil {
		logf.Println("ハッシュ対象ファイルの読み込みに失敗しました。:", file)
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
			return nil, err
		}

		hasher.Write(buffer[:ret])

		progressInfo.sizeCount.Increment(uint64(ret))

		progressInfoChannel <- progressInfo
	}

	return hasher.Sum(nil), nil
}
