package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/danielgtaylor/huma/v2"
	"github.com/danielgtaylor/huma/v2/adapters/humachi"
	"github.com/danielgtaylor/huma/v2/humacli"
	"github.com/go-chi/chi/v5"

	configModule "github.com/denwong47/rockyou2024/src/host/config"
	"github.com/denwong47/rockyou2024/src/host/consts"
	"github.com/denwong47/rockyou2024/src/host/index"
	"github.com/denwong47/rockyou2024/src/host/interfaces"
)

func main() {
	// Create a CLI app which takes a port option.
	cli := humacli.New(func(hooks humacli.Hooks, options *configModule.Options) {
		// TODO - Implement CLI flags for setting the port and other options
		router := chi.NewMux()
		config := huma.DefaultConfig("PasswordDumpSearch", "0.1.0")
		// config.Components.SecuritySchemes = map[string]*huma.SecurityScheme{
		// 	"BearerAuth": {
		// 		Type:         "http",
		// 		Scheme:       "bearer",
		// 		BearerFormat: "base64",
		// 	},
		// }
		api := humachi.New(router, config)
		api.OpenAPI().Info.Title = "Password Dump Search API"
		api.OpenAPI().Info.Description = consts.AppDescription + "\n\n" + consts.AppDisclaimer
		api.OpenAPI().Info.Contact = &huma.Contact{
			Name:  "Denny Wong",
			Email: "denwong47@hotmail.com",
			URL:   "https://github.com/denwong47/pigeon-hole",
		}
		log.Printf("Starting Query service...\n")
		log.Printf("Search operations will be limited to %v.\n", options.Timeout)

		huma.Register(api, huma.Operation{
			Method:      http.MethodGet,
			Path:        "/search",
			Summary:     "Search",
			Description: `Search for passwords in the RockYou2024 dataset.`,
			Errors:      []int{200, 400, 408, 422},
		}, interfaces.HostErrorWrapper(interfaces.Query))

		server := http.Server{
			Addr:    fmt.Sprintf("%s:%d", options.Host, options.Port),
			Handler: router,
		}

		if exists, err := index.Exists(configModule.DefaultIndexPath); exists {
			log.Printf("Index found at %s\n", configModule.DefaultIndexPath)
		} else if err == nil {
			log.Fatalf("Index not found at %s\n", configModule.DefaultIndexPath)
		} else {
			log.Fatalf("Could not use index at %s: %v\n", configModule.DefaultIndexPath, err)
		}

		hooks.OnStart(func() {
			if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
				log.Fatalf("Failed to start server: %v", err)
			}
		})

		hooks.OnStop(func() {
			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if err := server.Shutdown(ctx); err != nil {
				log.Fatalf("Failed to stop server: %v", err)
			}
		})
	})

	cli.Run()
}
