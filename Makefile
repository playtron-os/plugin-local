NAME := $(shell grep 'name =' Cargo.toml | head -n 1 | cut -d'"' -f2)
VERSION := $(shell grep '^version =' Cargo.toml | cut -d'"' -f2)
BUILD_TYPE ?= release
ARCH ?= $(shell uname -m)
TARGET_ARCH ?= $(ARCH)-unknown-linux-gnu
ALL_RS := $(shell find src -name '*.rs')
PREFIX ?= /usr
CACHE_DIR := .cache

# Docker image variables
IMAGE_NAME ?= plugin-builder
IMAGE_TAG ?= latest

##@ General

# The help target prints out all targets with their descriptions organized
# beneath their categories. The categories are represented by '##@' and the
# target descriptions by '##'. The awk commands is responsible for reading the
# entire set of makefiles included in this invocation, looking for lines of the
# file as xyz: ## something, and then pretty-format the target and help. Then,
# if there's a line with ##@ something, that gets pretty-printed as a category.
# More info on the usage of ANSI control characters for terminal formatting:
# https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters
# More info on the awk command:
# http://linuxcommand.org/lc3_adv_awk.php

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: install
install: build ## Install plugin to the given prefix (default: PREFIX=/usr)
	install -D -m 755 target/$(TARGET_ARCH)/$(BUILD_TYPE)/$(NAME) \
		$(PREFIX)/bin/$(NAME)
	install -D -m 644 ./pluginmanifest.json \
		$(PREFIX)/share/playtron/plugin/local/pluginmanifest.json

.PHONY: uninstall
uninstall: ## Uninstall plugin
	rm $(PREFIX)/bin/$(NAME)
	rm $(PREFIX)/share/playtron/plugin/local/pluginmanifest.json

##@ Development

.PHONY: build ## Build (Default: BUILD_TYPE=release)
build: target/$(TARGET_ARCH)/$(BUILD_TYPE)/$(NAME)

.PHONY: debug
debug: target/$(TARGET_ARCH)/debug/$(NAME)  ## Build debug build
target/$(TARGET_ARCH)/debug/$(NAME): $(ALL_RS) Cargo.lock
	cargo build --target $(TARGET_ARCH)

.PHONY: release
release: target/$(TARGET_ARCH)/release/$(NAME) ## Build release build
target/$(TARGET_ARCH)/release/$(NAME): $(ALL_RS) Cargo.lock
	cargo build --release --target $(TARGET_ARCH)

.PHONY: all
all: build debug ## Build release and debug builds

.PHONY: run
run: debug ## Build and run
	./target/$(TARGET_ARCH)/debug/$(NAME)

.PHONY: test
test: debug ## Build and run all tests
	cargo test -- --show-output

.PHONY: clean
clean: ## Remove build artifacts
	rm -rf target
	rm -rf .cache
	rm -rf dist

.PHONY: format
format: ## Run rustfmt on all source files
	rustfmt --edition 2021 $(ALL_RS)


##@ Distribution

.PHONY: dist
dist: dist/$(NAME)-$(ARCH).tar.gz dist/$(NAME)-$(VERSION)-1.$(ARCH).rpm ## Create all redistributable versions of the project

.PHONY: dist-archive
dist-archive: dist/$(NAME)-$(ARCH).tar.gz ## Build a redistributable archive of the project
dist/$(NAME)-$(ARCH).tar.gz: build
	rm -rf $(CACHE_DIR)/$(NAME)
	mkdir -p $(CACHE_DIR)/$(NAME)
	$(MAKE) install PREFIX=$(CACHE_DIR)/$(NAME)/usr
	mkdir -p dist
	tar cvfz $@ -C $(CACHE_DIR) $(NAME)
	cd dist && sha256sum $(NAME)-$(ARCH).tar.gz > $(NAME)-$(ARCH).tar.gz.sha256.txt

.PHONY: dist-rpm
dist-rpm: dist/$(NAME)-$(VERSION)-1.$(ARCH).rpm ## Build a redistributable RPM package
dist/$(NAME)-$(VERSION)-1.$(ARCH).rpm: target/$(TARGET_ARCH)/release/$(NAME)
	mkdir -p dist
	cargo install --version 0.16.1 cargo-generate-rpm
	cargo generate-rpm --target $(TARGET_ARCH)
	cp ./target/$(TARGET_ARCH)/generate-rpm/$(NAME)-$(VERSION)-1.$(ARCH).rpm dist
	cd dist && sha256sum $(NAME)-$(VERSION)-1.$(ARCH).rpm > $(NAME)-$(VERSION)-1.$(ARCH).rpm.sha256.txt

# Refer to .releaserc.yaml for release configuration
.PHONY: sem-release 
sem-release: ## Publish a release with semantic release 
	npx semantic-release

# Build the docker container for running in docker
.PHONY: docker-builder
docker-builder:
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

# E.g. make in-docker TARGET=build
.PHONY: in-docker
in-docker: docker-builder
	@# Run the given make target inside Docker
	docker run --rm \
		-v $(shell pwd):/src \
		--workdir /src \
		-e HOME=/home/build \
		-e ARCH=$(ARCH) \
		-e BUILD_TYPE=$(BUILD_TYPE) \
		-e PKG_CONFIG_SYSROOT_DIR="/usr/$(ARCH)-linux-gnu" \
		--user $(shell id -u):$(shell id -g) \
		$(IMAGE_NAME):$(IMAGE_TAG) \
		make $(TARGET)
