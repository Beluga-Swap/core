.PHONY: integration

integration:
	@echo "Running local integration tests..."
	@chmod +x tests/integration_factory/test_factory_local.sh
	@cd tests/integration_factory && ./test_factory_local.sh

integration-testnet:
	@chmod +x tests/integration_factory/test_factory_integration.sh
	@cd tests/integration_factory && ./test_factory_integration.sh

fuzz:
	@cd contracts/factory && PROPTEST_CASES=100 cargo test --test test_fuzz_factory

test:
	@cd contracts/factory && cargo test
	@cd contracts/pool && cargo test
