.PHONY: prepare prepare-backend prepare-frontend check check-ci check-quiet format format-check test test-ci audit backend frontend cargo-update generate-openapi

REPO_ROOT := $(shell git rev-parse --show-toplevel)

prepare:
	./configs/git/setup.sh
	make prepare-backend
	make prepare-frontend
prepare-backend:
	cd modules/backend && cargo sqlx prepare
prepare-frontend:
	cd modules/frontend && yarn prepare
backend:
	cd modules/backend && cargo run
frontend:
	cd modules/frontend && yarn dev
check:
	cd modules/backend && cargo clippy --all-features --all-targets --quiet
	cd modules/frontend && yarn check
format:
	cd modules/backend && cargo fmt
	cd modules/frontend && yarn format
format-check:
	cd modules/backend && $(REPO_ROOT)/configs/scripts/cargo-fmt.sh
test:
	#cd modules/backend && cargo test && cargo test -- --ignored # add virtual postgres for stable API tests
	cd modules/backend && cargo test
test-ci:
	# reduce number of tests run in parallel to avoid DB pool exhaustion
	cd modules/backend && cargo test -- --test-threads=1
audit:
	cd modules/backend && cargo audit
	cd modules/frontend && yarn npm audit
cargo-update:
	cd modules/backend && cargo update
generate-openapi:
	cd modules/backend && cargo run --bin openapi-generator