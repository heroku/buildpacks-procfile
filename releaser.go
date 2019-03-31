package releaser

import (
	"io/ioutil"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v2"

	"github.com/buildpack/libbuildpack/layers"
	"github.com/buildpack/libbuildpack/logger"
)

type Release struct {
	DefaultProcessTypes map[string]string `yaml:"default_process_types,omitempty"`
}

func WriteLaunchMetadata(appDir, layersDir string, log logger.Logger) (layers.Processes, error) {
	processes := layers.Processes{}

	procfile, err := ReadProcfile(appDir)
	if err != nil {
		return processes, err
	}

	processTypes := make(map[string]string)
	for name, command := range procfile {
		processTypes[name] = command
	}

	for name, command := range processTypes {
		processes = append(processes, layers.Process{
			Type:    name,
			Command: command,
		})
	}

	l := layers.NewLayers(layersDir, log)

	return processes, l.WriteApplicationMetadata(layers.Metadata{
		Processes: processes,
	})
}

func ReadProcfile(appDir string) (map[string]string, error) {
	processTypes := make(map[string]string)
	procfile := filepath.Join(appDir, "Procfile")
	_, err := os.Stat(procfile)
	if !os.IsNotExist(err) {
		procfileText, err := ioutil.ReadFile(procfile)
		if err != nil {
			return processTypes, err
		}

		return processTypes, yaml.Unmarshal(procfileText, &processTypes)
	} else {
		return processTypes, nil
	}
}