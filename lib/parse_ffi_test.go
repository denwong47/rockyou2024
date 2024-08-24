package libparseFfi

import (
	"slices"
	"testing"
)

func TestIndexOf(t *testing.T) {
	tests := []struct {
		item     string
		len      int
		expected []string
	}{
		{
			item: "password",
			len:  2,
			expected: []string{
				"pas",
				"wor",
			},
		},
		{
			item: "my密碼",
			len:  1,
			expected: []string{
				"my1",
			},
		},
		{
			item: "This is a big long sentence with a lot of words in it",
			len:  10,
			expected: []string{
				"thi",
				"sis",
				"abi",
				"ong",
				"sen",
				"ten",
				"wit",
				"hai",
				"wor",
				"sin",
			},
		},
	}

	for _, test := range tests {
		indices := IndexOf(test.item)

		if len(indices) != test.len {
			t.Errorf("For '%s', expected length of %d, got %d.", test.item, test.len, len(indices))
		}

		if !slices.Equal(indices, test.expected) {
			t.Errorf("For '%s', expected %v, got %v.", test.item, test.expected, indices)
		}
	}
}
