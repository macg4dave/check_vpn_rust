IMAGE_NAME := check_vpn_tests

.PHONY: test-container test-container-ignored

# Build and run the test container (runs default tests)
test-container:
	docker build -t $(IMAGE_NAME) -f contrib/Dockerfile .
	docker run --rm $(IMAGE_NAME)

# Build and run the test container but execute ignored tests (real-network)
test-container-ignored:
	docker build -t $(IMAGE_NAME) -f contrib/Dockerfile .
	docker run --rm $(IMAGE_NAME) sh -c "cargo test -- --ignored"
