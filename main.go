package main

import (
	"log"

	libparseFfi "github.com/denwong47/rockyou2024/lib"
)

func main() {
	indices := libparseFfi.IndexOf("This is a big long sentence with a lot of words in it")
	for _, item := range indices {
		log.Println(item)
	}
}
