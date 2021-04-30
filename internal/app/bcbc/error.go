package bcbc

// 引数errorOccuredがtrueなら引数messageをログ出力してプログラムを終了する。
func fatalMessageIf(errorOccurred bool, format string, values ...interface{}) {
	if errorOccurred {
		logf.Fatalf(format, values...)
	}
}

// 引数errorOccuredがtrueなら引数messageをログ出力してプログラムを終了する。
func fatalMessageError(err error, format string, values ...interface{}) {
	if err != nil {
		logf.Printf(format, values...)
		logf.Fatalln(err)
	}
}
