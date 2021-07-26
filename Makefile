.EXPORT_ALL_VARIABLES:

.PHONY: test \
        build \
        clean \
        package \
        release

SHELL=/bin/bash -o pipefail

GO111MODULE := on

VERSION := "v$$(cat buildpack.toml | grep version | sed -e 's/version = //g' | xargs)"

build:
	@GOOS=linux go build -o "bin/release" ./cmd/release/...

test:
	go test ./... -v

clean:
	-rm -f procilfe-cnb-$(VERSION).tgz
	-rm -f bin/release

package: clean build
	@pack buildpack package --format=file heroku_procfile_$(VERSION).cnb

release:
	@git tag $(VERSION)
	@git push --tags origin master
