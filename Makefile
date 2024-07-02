.PHONY: code-coverage
code-coverage:
	cargo tarpaulin --out html
