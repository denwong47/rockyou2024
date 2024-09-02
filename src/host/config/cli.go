package config

import "time"

// Options for the CLI.
type Options struct {
	Host    string        `doc:"Host to listen on" format:"ipv4" default:"0.0.0.0"`
	Port    int           `doc:"Port to listen on" short:"p" default:"8888"`
	Timeout time.Duration `doc:"Timeout for requests in seconds" default:"15s"`
}
