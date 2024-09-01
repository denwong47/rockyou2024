package interfaces

import (
	"context"
	"log"
	"time"

	"github.com/denwong47/rockyou2024/src/host/config"
	errorMessages "github.com/denwong47/rockyou2024/src/host/errors"
	index "github.com/denwong47/rockyou2024/src/host/index"
)

// Query is the main query endpoint.
func Query(
	ctx context.Context,
	cache *index.CacheType,
	input *QueryRequest,
) (*QueryResponse, errorMessages.HostError) {
	if input.Style == "" {
		input.Style = index.SearchStyle("fuzzy")
	}

	log.Printf("Searching for '%s' with style '%s'...", input.Query, input.Style)

	if results, err := index.FindLinesInIndexCollectionPaginated(
		config.DefaultIndexPath,
		input.Query,
		input.Style,
		input.Offset,
		input.Limit,
		cache,
	); !err.IsEmpty() {
		return &QueryResponse{}, err
	} else {
		return &QueryResponse{
			Body: QueryReponseBody{
				Query:     input,
				Results:   results,
				Timestamp: time.Now(),
			},
		}, errorMessages.EmptyError()
	}
}
