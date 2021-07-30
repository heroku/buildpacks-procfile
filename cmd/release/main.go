package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/heroku/procfile-cnb"
)

func main() {
	if len(os.Args) != 4 {
		fmt.Println("Usage:", os.Args[0], "APP_DIR", "LAYERS_DIR", "PLATFORM_DIR")
		return
	}

	println("[INFO] Discovering process types")
	appDir := os.Args[1]
	layersDir := os.Args[2]

	processes, err := releaser.WriteLaunchMetadata(appDir, layersDir)
	if err != nil {
		print(err.Error())
		os.Exit(3)
	}

	processNames := []string{}
	if len(processes) > 0 {
		for _, process := range processes {
			processNames = append(processNames, process.Type)
		}
	} else {
		processNames = append(processNames, "(none)")
	}

	fmt.Printf("[INFO] Procfile declares types -> %s\n", strings.Join(processNames, ", "))
}
