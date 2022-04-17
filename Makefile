VERSION=0.8.0

doc:
	cargo doc

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

release: doc test fmt clippy build deploy
	cargo build --release

build:
	cargo build --release

clean:
	sudo rm -r /usr/local/bin/apish

test-release: clean build deploy

deploy:
	sudo cp target/release/apish /usr/local/bin/apish

docker-build:
	docker build -t=apish:$(VERSION) .

docker-smoke:
	docker run --rm -v $PWD:/test apish:$(VERSION) -e /test/example.json -f /test/src/example.api -o /test/api.json -s /test/api-spec.json

docker-tag:
	docker tag apish:$(VERSION) apish:latest
	docker tag apish:$(VERSION) elfidomx/apish:$(VERSION)

docker-publish: docker-build docker-smoke docker-tag
	docker push elfidomx/apish:$(VERSION)