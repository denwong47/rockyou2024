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
	input *QueryRequest,
) (*QueryResponse, errorMessages.HostError) {
	if input.Style == "" {
		input.Style = index.SearchStyle("fuzzy")
	}

	log.Printf("Searching for '%s' with style '%s'\n...", input.Query, input.Style)

	if results, err := index.FindLinesInIndexCollection(
		config.DefaultIndexPath,
		input.Query,
		input.Style,
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
