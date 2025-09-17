dev:
	concurrently \
	"cargo watch -x 'run -- serve' -w theme -w zap.toml" \
	"tailwindcss -i ./theme/style.css -o ./out/style.css --watch"

lint:
	cargo clippy

lint-fix:
	cargo clippy --fix