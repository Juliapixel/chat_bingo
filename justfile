import? "local.just"

_default:
    just --list

alias bb := build_backend
build_backend profile="dev":
    cargo build --manifest-path ./backend/Cargo.toml --profile {{profile}}

alias rb := run_backend
run_backend profile="dev" *ARGS="":
    cd ./backend && cargo run --profile {{profile}} -- {{ARGS}}

alias tb := test_backend
test_backend $RUST_BACKTRACE="1":
    cargo test --manifest-path ./backend/Cargo.toml

alias cb := clean_backend
clean_backend:
    cargo clean --manifest-path ./backend/Cargo.toml

build_backend_image:
    cd ./backend && docker build -t bingo_backend .

alias bw := build_web
build_web:
    cd ./blazor_wasm/bingo_frontend_wasm && dotnet build

alias rw := run_web
run_web:
    cd ./blazor_wasm/bingo_frontend_wasm && dotnet run
