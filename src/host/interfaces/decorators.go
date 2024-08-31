package interfaces

import (
	"context"

	errorMessages "github.com/denwong47/rockyou2024/src/host/errors"
)

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
