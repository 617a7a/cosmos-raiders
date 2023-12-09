r:
    cargo run --features bevy/dynamic_linking -p cosmos-raiders -- --skip-menu

rr:
    cargo run -p cosmos-raiders -r

w:
    cargo watch -x 'run --features bevy/dynamic_linking -p cosmos-raiders -- --skip-menu'

wr:
    cargo watch -x 'run -p cosmos-raiders -r'
