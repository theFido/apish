VERSION=0.7.1

doc:
	cargo doc

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

release: doc test fmt clippy
	cargo build --release

docker-build:
	docker build -t=apish:$(VERSION) .

docker-smoke:
	docker run --rm -v $PWD:/test apish:$(VERSION) -e /test/example.json -f /test/src/example.api -o /test/api.json -s /test/api-spec.json

docker-tag:
	docker tag apish:$(VERSION) apish:latest
	docker tag apish:$(VERSION) elfidomx/apish:$(VERSION)

docker-publish: docker-build docker-smoke docker-tag
	docker push elfidomx/apish:$(VERSION)