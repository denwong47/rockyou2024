package index

import (
	"slices"
	"testing"
)

// getTestBuffer returns a test buffer.
func getTestBuffer() []byte {
	return []byte(
		`somePassword
		passssssword
		MyPassword
		FooBarBaz`,
	)
}

// WIP - This is not gonna work - this Aho Corsick implementation only says
// found or not found, not the index.
func TestFindMathes(t *testing.T) {
	buffer := getTestBuffer()

	tests := []struct {
		keywords []string
		expected []int
	}{
		{
			keywords: []string{"Pass"},
			expected: []int{0},
		},
	}

	for _, test := range tests {
		matcher := BuildMatcher(test.keywords)

		matches := FindMatches(buffer, matcher)

		if !slices.Equal(matches, test.expected) {
			t.Errorf("For '%v', expected %v, got %v.", test.keywords, test.expected, matches)
		}
	}
}
