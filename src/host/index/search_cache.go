package index

import (
	"errors"
	"log"

	errorMessages "github.com/denwong47/rockyou2024/src/host/errors"
	lru "github.com/hashicorp/golang-lru/v2"
)

// The key for the cache.
type CacheKey struct {
	Query string
	Style SearchStyle
}

// An alias for the LRU cache type.
type CacheType = lru.Cache[CacheKey, []string]

// `NewCache` creates a new LRU cache with the specified size.
func NewCache(size int) (*CacheType, errorMessages.HostError) {
	if cache, err := lru.New[CacheKey, []string](size); err != nil {
		return nil, errorMessages.FromError(
			err,
			"Failed to create a new cache.",
			500,
			nil,
		)
	} else {
		return cache, errorMessages.EmptyError()
	}
}

// Find the lines in the index collection, and cache the results.
func FindLinesInIndexCollectionCached(dir string, query string, style SearchStyle, cache *CacheType) ([]string, errorMessages.HostError) {
	searchString := QueryAsSearchString(query, style)

	// Check if the cache contains the query.
	if result, ok := cache.Get(CacheKey{Query: searchString, Style: style}); ok {
		log.Printf("Cache hit for query `%s` using `%+v`, returning %d results.", searchString, style, len(result))
		return result, errorMessages.EmptyError()
	}

	results, err := FindLinesInIndexCollection(dir, query, style)

	// If there was an error, return the error.
	if !err.IsEmpty() {
		return nil, err
	}

	// Add the results to the cache.
	if cache.Add(CacheKey{Query: searchString, Style: style}, results) {
		log.Printf("Added query `%s` to the cache using `%+v`, which caused an eviction.", searchString, style)
	}

	return results, err
}

// Find the lines in the index collection, paginate the results, and return them.
func FindLinesInIndexCollectionPaginated(dir string, query string, style SearchStyle, offset int, limit int, cache *CacheType) ([]string, errorMessages.HostError) {
	if results, err := FindLinesInIndexCollectionCached(dir, query, style, cache); !err.IsEmpty() {
		return nil, err
	} else {
		// Paginate the results.
		start := offset
		end := offset + limit

		if start >= len(results) {
			return nil, errorMessages.FromError(
				errors.New("reached the end of results"),
				"Reached the end of results; try lowering the offset.",
				404,
				nil,
			)
		}

		// If the end index is greater than the length of the results, set it to the length of the results.
		if end > len(results) {
			end = len(results)
		}

		return results[start:end], err
	}
}
