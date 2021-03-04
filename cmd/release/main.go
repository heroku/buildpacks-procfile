package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/buildpack/libbuildpack/logger"
	"github.com/heroku/procfile-cnb"
)

func main() {
	if len(os.Args) != 4 {
		fmt.Println("Usage:", os.Args[0], "APP_DIR", "LAYERS_DIR", "PLATFORM_DIR")
		return
	}

	log, err := logger.DefaultLogger(os.Args[3])
	if err != nil {
		log.Info(err.Error())
		os.Exit(1)
	}

	log.Info("[INFO] Discovering process types")
	appDir := os.Args[1]
	layersDir := os.Args[2]

	processes, err := releaser.WriteLaunchMetadata(appDir, layersDir, log)
	if err != nil {
		log.Info(err.Error())
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

	log.Info("[INFO] Procfile declares types -> %s", strings.Join(processNames, ", "))
}
