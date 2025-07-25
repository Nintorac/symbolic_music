.PHONY: build run clean help

# Variables
IMAGE_NAME = aria-test-suite
OUTPUT_DIR = ../tests/assets/aria_targets

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

build: ## Build the Docker image
	docker build -t $(IMAGE_NAME) .

run: build ## Build and run the container to generate test data
	docker run --rm -v $(shell pwd)/$(OUTPUT_DIR):/workspace/output $(IMAGE_NAME)
	sudo chown -R $(shell id -u):$(shell id -g) $(OUTPUT_DIR)

clean: ## Remove the Docker image
	docker rmi $(IMAGE_NAME) 2>/dev/null || true

rebuild: clean build ## Clean and rebuild the Docker image

# Run without rebuilding (useful for development)
run-only: ## Run the container without rebuilding
	docker run --rm -v $(shell pwd)/$(OUTPUT_DIR):/workspace/output $(IMAGE_NAME)
	sudo chown -R $(shell id -u):$(shell id -g) $(OUTPUT_DIR)