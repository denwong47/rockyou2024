package libparseFfi

import (
	"slices"
	"testing"
)

const TestMockIndex = "../.tests/mock_index/"

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

func TestQueryAsSearchString(t *testing.T) {
	tests := []struct {
		query       string
		searchStyle SearchStyle
		expected    string
	}{
		{
			query:       "password",
			searchStyle: StrictSearch,
			expected:    "password",
		},
		{
			query:       "Password",
			searchStyle: StrictSearch,
			expected:    "Password",
		},
		{
			query:       "P455 w0rd",
			searchStyle: StrictSearch,
			expected:    "P455 w0rd",
		},
		{
			query:       "Password",
			searchStyle: CaseInsensitiveSearch,
			expected:    "password",
		},
		{
			query:       "P455 w0rd",
			searchStyle: CaseInsensitiveSearch,
			expected:    "p455 w0rd",
		},
		{
			query:       "P455 w0rd",
			searchStyle: FuzzySearch,
			expected:    "pass word",
		},
	}

	for _, test := range tests {
		searchString := QueryAsSearchString(test.query, test.searchStyle)

		if searchString != test.expected {
			t.Errorf("For '%s', expected %s, got %s.", test.query, test.expected, searchString)
		}
	}
}

func TestFindLinesInIndexCollection(t *testing.T) {
	tests := []struct {
		query       string
		searchStyle SearchStyle
		expected    []string
	}{
		{
			query:       "password",
			searchStyle: StrictSearch,
			expected: []string{
				"mypassword",
				"mapassword",
				"password13",
				"passwordz",
				"password5",
				"password75",
				"password1992",
				"password12",
				"password",
				"password1994",
				"password1!",
				"$password$",
				"password2",
				"!password!",
				"password123",
				"passwords",
				"xpasswordx",
				"password4",
				"(password)",
				"password3",
				"password.",
				"0password0",
				"**password**",
				"password11",
				"1password",
				"password7",
				"password!",
				"thisispassword",
				"password1",
				"thispassword",
			},
		},
	}

	for _, test := range tests {
		lines := FindLinesInIndexCollection(TestMockIndex, test.query, test.searchStyle)

		if !slices.Equal(lines, test.expected) {
			t.Errorf("For '%s', expected %v, got %v.", test.query, test.expected, lines)
		}
	}
}
