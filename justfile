alias b := build

build:
    cargo build

watch:
    cargo watch --no-gitignore --ignore output -x 'run -- generate -i example-project/input -o example-project/output -d'

