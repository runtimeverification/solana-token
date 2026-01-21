TOP_DIR := $(shell pwd)

stable-mir-json: CARGO_BUILD_OPTS =
stable-mir-json:
	cd deps/stable-mir-json && cargo build ${CARGO_BUILD_OPTS}
	cd deps/stable-mir-json && cargo run --bin cargo_stable_mir_json -- ${TOP_DIR}/deps/stable-mir-json ${TOP_DIR}/deps
	${TOP_DIR}/deps/.stable-mir-json/release.sh --version || ${TOP_DIR}/deps/.stable-mir-json/debug.sh --version
