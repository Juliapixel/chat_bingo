_default:
    just --list

alias bb := build_backend
build_backend profile="dev":
    cargo build --manifest-path ./backend/Cargo.toml --profile {{profile}}

alias rb := run_backend
run_backend profile="dev" args="":
    cargo run --manifest-path ./backend/Cargo.toml --profile {{profile}} -- {{args}}

alias bw := build_web
build_web:
    cd ./web && npm install && npm run build

alias rw := run_web
run_web:
    cd ./web && npm install && npm run dev
