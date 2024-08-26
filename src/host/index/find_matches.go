package index

import (
	"github.com/cloudflare/ahocorasick"
)

// BuildMathcer creates a new Aho-Corasick matcher with the given keywords.
func BuildMatcher(keywords []string) *ahocorasick.Matcher {
	return ahocorasick.NewStringMatcher(keywords)
}

// FindMatches returns the start and end indices of all matches of the given keywords in the buffer.
func FindMatches(buffer []byte, matcher *ahocorasick.Matcher) []int {
	return matcher.MatchThreadSafe(buffer)
}

// FindLineOfMatch returns the line of the buffer that contains the match.
func FindLineOfMatch(buffer []byte, start int, end int) []byte {
	for start > 0 {
		if buffer[start] == '\n' {
			start++
			break
		}
		start--
	}

	for end < len(buffer) {
		if buffer[end] == '\n' {
			end--
			break
		}
		end++
	}

	return buffer[start:end]
}
