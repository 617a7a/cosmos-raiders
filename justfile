export MTL_HUD_ENABLED := "1"

# Run in debug mode
r:
    cargo run --features bevy/dynamic_linking -p cosmos-raiders -- --skip-menu

# Run in release mode
rr:
    cargo run -p cosmos-raiders -r

# Run in debug mode with cargo-watch
w:
    cargo watch -x 'run --features bevy/dynamic_linking -p cosmos-raiders -- --skip-menu'

# Run in release mode with cargo-watch
wr:
    cargo watch -x 'run -p cosmos-raiders -r'

fmt:
    cargo fmt
