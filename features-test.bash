#!/bin/bash

# sane error handling
set -eEuo pipefail

# default
cargo test

# no_std / no deps
cargo test --no-default-features

# no_std / serde
cargo test --no-default-features --features serde

# std / serde
cargo test --no-default-features --features serde,std,serde_std

# std / futures-io
cargo test --no-default-features --features std,futures-io

# std / tokio
cargo test --no-default-features --features std,tokio

# std / tokio + futures-io (make sure they don't fight...)
cargo test --no-default-features --features std,tokio,futures-io

# std / tokio / serde
cargo test --no-default-features --features std,tokio,serde,serde_std
