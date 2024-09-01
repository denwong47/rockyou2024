package index

import (
	"errors"
	"net/http"
	"os"
	"strings"

	libparseFfi "github.com/denwong47/rockyou2024/lib"
	errorMessages "github.com/denwong47/rockyou2024/src/host/errors"
)

// Re-export `SearchStyle`, an enum for the search style.
type SearchStyle = libparseFfi.SearchStyle

// Exists checks if a directory exists.
func Exists(dir string) (bool, error) {
	entries, err := os.ReadDir(dir)

	if err == nil {
		for _, entry := range entries {
			if strings.HasSuffix(entry.Name(), ".csv") {
				return true, nil
			}
		}
		return false, errors.New("index directory was found, but it contains no index (CSV) files")
	}
	if os.IsNotExist(err) {
		return false, nil
	} else {
		// If there was an error that was not a "does not exist" error, return the error.
		return false, err
	}
}

// Re-export the `QueryAsSearchString` function from the `libparseFfi` package,
// to make it more ergonomic to use.
func QueryAsSearchString(query string, style SearchStyle) string {
	return libparseFfi.QueryAsSearchString(query, style)
}

// Re-export the `FindLinesInIndexCollection` function from the `libparseFfi` package,
// to make it more ergonomic to use.
func FindLinesInIndexCollection(dir string, query string, style SearchStyle) ([]string, errorMessages.HostError) {
	results := libparseFfi.FindLinesInIndexCollection(dir, query, style)

	if len(results) == 0 {
		return nil, errorMessages.FromError(
			errors.New("`libparseFfi.FindLinesInIndexCollection` returned no results"),
			"No results found; or an error occurred during the search. Please consult the logs for more information.",
			http.StatusNotFound,
			nil,
		)
	}

	return results, errorMessages.EmptyError()
}
