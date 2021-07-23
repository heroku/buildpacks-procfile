package releaser

import (
	"io/ioutil"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
	"gopkg.in/yaml.v2"

	cnb "github.com/buildpacks/libcnb"
)

type Release struct {
	DefaultProcessTypes map[string]string `yaml:"default_process_types,omitempty"`
}

func WriteLaunchMetadata(appDir, layersDir string) ([]cnb.Process, error) {
	processes := []cnb.Process{}

	procfile, err := ReadProcfile(appDir)
	if err != nil {
		return processes, err
	}

	processTypes := make(map[string]string)
	for name, command := range procfile {
		processTypes[name] = command
	}

	for name, command := range processTypes {
		processes = append(processes, cnb.Process{
			Type:    name,
			Command: command,
			Default: name == "web",
		})
	}

	err = os.MkdirAll(layersDir, 0755)
	if err != nil {
		return processes, err
	}

	launch := cnb.LaunchTOML{
		Processes: processes,
	}

	f, err := os.Create(filepath.Join(layersDir, "launch.toml"))
	if err != nil {
		return processes, err
	}
	defer f.Close()

	return processes, toml.NewEncoder(f).Encode(launch)
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
