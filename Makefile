.PHONY: help lint lint-fix test test-e2e check check-rust check-frontend pre-push clean

# Default target
help:
	@echo "TracSeq 2.0 Development Commands"
	@echo "================================"
	@echo ""
	@echo "Frontend Commands:"
	@echo "  make lint            - Run ESLint on frontend code"
	@echo "  make lint-fix        - Run ESLint and fix issues"
	@echo "  make test            - Run frontend unit tests"
	@echo "  make test-e2e        - Run Playwright E2E tests"
	@echo "  make check-frontend  - Run all frontend checks"
	@echo ""
	@echo "Rust Commands:"
	@echo "  make check-rust      - Run cargo check, fmt, and clippy"
	@echo "  make fmt             - Format Rust code"
	@echo "  make test-rust       - Run Rust tests"
	@echo ""
	@echo "Combined Commands:"
	@echo "  make check           - Run all checks (frontend + Rust)"
	@echo "  make pre-push        - Run pre-push checks"
	@echo "  make clean           - Clean build artifacts"

# Frontend commands
lint:
	cd lims-ui && pnpm lint

lint-fix:
	cd lims-ui && pnpm lint --fix

test:
	cd lims-ui && pnpm test --passWithNoTests

test-e2e:
	cd lims-ui && pnpm test:e2e

check-frontend: lint test
	@echo "✅ Frontend checks completed"

# Rust commands
check-rust:
	cargo check --workspace --all-targets
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings || true

fmt:
	cargo fmt --all

test-rust:
	cargo test --workspace

# Combined commands
check: check-frontend check-rust
	@echo "✅ All checks completed"

pre-push:
	./scripts/pre-push-check.sh

# Clean build artifacts
clean:
	cargo clean
	cd lims-ui && rm -rf dist node_modules .turbo
	find . -name "*.log" -type f -delete
	find . -name ".DS_Store" -type f -delete

# Development helpers
dev-frontend:
	cd lims-ui && pnpm dev

dev-rust:
	./scripts/run_full_app.sh

# Docker helpers
docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

# Kubernetes helpers
k8s-deploy:
	kubectl apply -f infrastructure/kubernetes/namespace.yaml
	helm install tracseq infrastructure/kubernetes/helm/tracseq -n tracseq-dev

k8s-status:
	kubectl get all -n tracseq-dev

k8s-logs:
	kubectl logs -n tracseq-dev -l app.kubernetes.io/name=tracseq -f

# Database helpers
db-migrate:
	for service in lims-core/*/migrations; do \
		if [ -d "$$service" ]; then \
			echo "Running migrations for $$service..."; \
			cd "$$(dirname $$service)" && sqlx migrate run || true; \
			cd -; \
		fi \
	done

db-reset:
	docker-compose exec -T postgres psql -U postgres -c "DROP DATABASE IF EXISTS tracseq;"
	docker-compose exec -T postgres psql -U postgres -c "CREATE DATABASE tracseq;"
	$(MAKE) db-migrate 