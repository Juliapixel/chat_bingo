_default:
    just --list

build_backend profile="dev":
    cargo build --manifest-path ./backend/Cargo.toml --profile {{profile}}

run_backend profile="dev" args="":
    cargo run --manifest-path ./backend/Cargo.toml --profile {{profile}} -- {{args}}

build_web:
    cd ./web && npm install && npm run build

run_web:
    cd ./web && npm install && npm run dev

alias bb := build_backend
alias rb := run_backend
alias bw := build_web
