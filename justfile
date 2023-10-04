r:
    cargo run --features bevy/dynamic_linking -p cosmos-raiders
    
rr:
    cargo run -p cosmos-raiders -r
    
w:
    cargo watch -x 'run --features bevy/dynamic_linking -p cosmos-raiders'

wr:
    cargo watch -x 'run -p cosmos-raiders -r'