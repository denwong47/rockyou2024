package libparseFfi

/*
#cgo LDFLAGS: -L. -lparse_ffi
#include "./parse_ffi.h"
#include <stdlib.h>
*/
import "C"

import (
	"log"
	"unsafe"
)

// Convert a C array of strings to a Go slice of strings.
func pointerToArray(ptr unsafe.Pointer) []string {
	if ptr == nil {
		log.Println("Received nil from C function")
		return make([]string, 0)
	}

	// Convert the C array of strings to a Go slice of strings
	var goStrings []string
	for i := 0; ; i++ {
		ptr := *(**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(ptr)) + uintptr(i)*unsafe.Sizeof(uintptr(0))))
		defer C.free(unsafe.Pointer(ptr))

		if ptr == nil {
			break
		}
		goStrings = append(goStrings, C.GoString(ptr))
	}
	defer C.free(unsafe.Pointer(ptr))

	return goStrings
}

// IndexOf returns the indices of the words in the input string.
//
// This internally calls the Rust function `indices_of` which returns a C array of strings,
// which is then converted to a Go slice of strings.
//
// This is to ensure that the indexing used for searching is the same as the one used to
// generate the dictionary in the first place.
func IndexOf(item string) []string {
	mystr := C.CString(item)
	defer C.free(unsafe.Pointer(mystr))

	indices := C.indices_of(mystr)

	return pointerToArray(unsafe.Pointer(indices))
}

// Define a custom type for your string literals
type SearchStyle string

// Define constants representing the allowed values
const (
	StrictSearch          SearchStyle = "strict"
	CaseInsensitiveSearch SearchStyle = "case-insensitive"
	FuzzySearch           SearchStyle = "fuzzy"
)

// FindLinesInIndexCollection returns the lines in the index collection that match the query.
func FindLinesInIndexCollection(dir string, query string, style SearchStyle) []string {
	dirStr := C.CString(dir)
	queryStr := C.CString(query)
	searchStyleStr := C.CString(string(style))

	lines := C.find_lines_in_index_collection(dirStr, queryStr, searchStyleStr)

	return pointerToArray(unsafe.Pointer(lines))
}
