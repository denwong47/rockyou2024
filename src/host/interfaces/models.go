package interfaces

import (
	"time"

	"github.com/denwong47/rockyou2024/src/host/index"
)

// QueryRequest is the request object the main query endpoint.
type QueryRequest struct {
	Query  string            `query:"query" required:"true" maxLength:"256" example:"myPassword" doc:"The password pattern to search for."`
	Style  index.SearchStyle `query:"style" doc:"The search style to use; allowed values are 'fuzzy', 'case-insensitive' and 'strict'. Defaults to 'fuzzy'." default:"fuzzy"`
	Offset int               `query:"offset" doc:"The offset to start the search from." default:"0"`
	Limit  int               `query:"limit" doc:"The maximum number of results to return." default:"500"`
}

// QueryReponseBody is the response body for the main query endpoint.
type QueryReponseBody struct {
	Query     *QueryRequest `json:"query" doc:"The query that was searched for."`
	Results   []string      `json:"results" doc:"The results of the search."`
	Timestamp time.Time     `json:"timestamp" doc:"The timestamp of the search."`
}

// QueryResponse is the response object for the main query endpoint.
type QueryResponse struct {
	Body QueryReponseBody `json:"body" doc:"The body of the response."`
}
