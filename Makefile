.PHONY: setup check fmt test-smoke test-full bench offload clean

# One-time environment bootstrap (Termux-aware). Safe to re-run.
setup:
	bash scripts/setup_env.sh

# Cheap correctness check — safe to run on any device, any time.
check:
	cargo check --all-targets
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt
	command -v black >/dev/null 2>&1 && black scripts matrix_tests || true

# Fast local build + small differential-test subset.
test-smoke:
	maturin develop
	python3 scripts/compare_outputs.py --size smoke

# Full release build + full differential-test matrix. Heavy — the verifier
# agent offloads this to CI on constrained devices; run by hand only if
# you know your machine can take it.
test-full:
	maturin develop --release
	python3 scripts/compare_outputs.py --size full

bench:
	python3 scripts/benchmark.py

# Manual equivalent of the ci-dispatcher agent: push + trigger GH Actions
# + wait + pull results back. Usage: make offload MODE=full
MODE ?= smoke
offload:
	bash scripts/dispatch_and_wait.sh $(MODE)

clean:
	cargo clean
	rm -rf __pycache__ matrix-report matrix_report.md matrix_report.json
