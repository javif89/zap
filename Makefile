dev:
	concurrently \
	"cargo watch -x 'run -p zap-cli' -w theme -w zap.toml" \
	"tailwindcss -i ./theme/style.css -o ./out/style.css --watch" \
	"npx live-server out"