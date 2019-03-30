.EXPORT_ALL_VARIABLES:

.PHONY: test \
        build \
        clean \
        package \
        release

SHELL=/bin/bash -o pipefail

GO111MODULE := on

VERSION := "v0.0.1"

build:
	@GOOS=linux go build -o "bin/release" ./cmd/release/...

test:
	go test ./... -v

clean:
	-rm -f procilfe-cnb-$(VERSION).tgz
	-rm -f bin/release

package: clean build
	@tar cvzf procfile-cnb-$(VERSION).tgz bin/ README.md LICENSE

release:
	@git tag $(VERSION)
	@git push --tags origin master
