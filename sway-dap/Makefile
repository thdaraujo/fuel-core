ASSETS = assets
SCHEMA := "debugAdapterProtocol.json"

.PHONY: help

schema: ## Update the JSON schema of DAP types
	@curl https://microsoft.github.io/debug-adapter-protocol/$(SCHEMA) \
		-H "Accept: application/json" \
		--output $(ASSETS)/$(SCHEMA) \
		-s
	@schemafy-cli $(ASSETS) | rustfmt | tee src/schema.rs

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'
