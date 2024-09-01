package interfaces

import (
	"context"

	errorMessages "github.com/denwong47/rockyou2024/src/host/errors"
	index "github.com/denwong47/rockyou2024/src/host/index"
)

// Short Hand for the EndpointHandler function signature.
type EndpointHandler[T, R, E any] func(ctx context.Context, input *T) (*R, E)

// EndpointHandler is a function that handles an endpoint.
type EndpointHandlerWithCache[T, R, E any] func(ctx context.Context, cache *index.CacheType, input *T) (*R, E)

// HostErrorWrapper wraps a function that returns a HostError and converts it to a
// standard error if it is not empty.
func HostErrorWrapper[Q, R any](
	fn func(ctx context.Context, input *Q) (*R, errorMessages.HostError),
) func(ctx context.Context, input *Q) (*R, error) {
	return func(ctx context.Context, input *Q) (*R, error) {
		if res, err := fn(ctx, input); !err.IsEmpty() {
			return res, err.Response()
		} else {
			return res, nil
		}
	}
}

// Decorator to transform a `EndpointHandlerWithAuthManager` into a `EndpointHandler`.
func UsesCache[T, R, E any](
	cache *index.CacheType,
	handler EndpointHandlerWithCache[T, R, E],
) EndpointHandler[T, R, E] {
	return func(ctx context.Context, input *T) (*R, E) {
		return handler(ctx, cache, input)
	}
}
