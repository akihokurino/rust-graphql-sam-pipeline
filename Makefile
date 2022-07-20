STAGE := release
RELEASE_FLAG := --release
ifeq ($(DEBUG), 1)
	STAGE := debug
	RELEASE_FLAG :=
endif

DOCKER_CMD_BASE :=
DOCKER_EXTRA_PARAMS :=
ifeq ($(USE_DOCKER), 1)
	DOCKER_CACHE_PARAMS :=
	ifeq ($(USE_DOCKER_CACHE), 1)
		DOCKER_CACHE_PARAMS := -v "$(shell pwd)/.docker/cache/cargo:/home/rust/.cargo"
	endif
	DOCKER_CMD_BASE := docker run --rm -v "$(shell pwd):/home/rust/src" $(DOCKER_CACHE_PARAMS) $(DOCKER_EXTRA_PARAMS) ekidd/rust-musl-builder
endif

TARGET_TRIPLE := x86_64-unknown-linux-musl
BIN_OUTDIR := target/$(TARGET_TRIPLE)/$(STAGE)
SRC_FILES := $(shell find . -type f | grep -v '^\./target' | grep -v '/\.')
DEPLOY_CRATES := api

clean:
	cargo clean
	rm -rf .aws-sam

run-api:
	SSM_PARAMETER=/rust-graphql-sam-sample/server/env cargo run --bin api

$(BIN_OUTDIR)/%: $(SRC_FILES)
	$(DOCKER_CMD_BASE) cargo build $(RELEASE_FLAG) --bin $(lastword $(subst /, ,$@)) --target $(TARGET_TRIPLE)
	if [ "$(STRIP)" = "1" ]; then strip $@; fi

.PHONY: build
build: $(addprefix $(BIN_OUTDIR)/,$(DEPLOY_CRATES))

build-ApiFunction: $(BIN_OUTDIR)/api
	cp $< $(ARTIFACTS_DIR)/bootstrap

build-debug:
	cargo build --bin api

.aws-sam: $(addprefix $(BIN_OUTDIR)/,$(DEPLOY_CRATES))
	sam build

deploy: .aws-sam
	sam deploy --no-confirm-changeset --no-fail-on-empty-changeset