/*
Package errorMessages contains all the error messages that are used in the application.
*/
package errorMessages

import (
	"github.com/danielgtaylor/huma/v2"
)

// A custom error type that can holds an error as well as its HTTP response information.
type HostError struct {
	Message string
	Errors  []error
	Code    int
	Headers map[string]string
}

// Create a new empty HostError, which is different from an unknown error below.
//
// Use this only for cases where an error had not occurred.
func EmptyError() HostError {
	return HostError{}
}

// Check if the HostError is empty.
func (he *HostError) IsEmpty() bool {
	return he.Message == "" && he.Errors == nil && he.Code == 0 && he.Headers == nil
}

// Create a new HostError from an error, status code, and headers.
func FromError(err error, message string, code int, headers map[string]string) HostError {
	if code < 100 || code > 599 {
		code = 500
	}

	if headers == nil {
		headers = make(map[string]string, 1)
	}

	if err == nil {
		if message == "" {
			message = "An unknown error occurred; no error was provided."
		}

		// This is the nil object pattern.
		return HostError{
			Message: message,
			Errors:  nil,
			Code:    code,
			Headers: headers,
		}
	}

	if message == "" {
		message = err.Error()
	}

	if _, ok := headers["Content-Type"]; !ok {
		headers["Content-Type"] = "application/json"
	}

	return HostError{
		Message: message,
		Errors:  []error{err},
		Code:    code,
		Headers: headers,
	}
}

// Add a new error to the HostError.
func (he *HostError) AddError(err error) {
	// If the error is nil, don't add it.
	if err != nil {
		he.Errors = append(he.Errors, err)
	}
}

func (he *HostError) Response() huma.StatusError {
	return huma.NewError(
		he.Code,
		he.Message,
		he.Errors...,
	)
}
