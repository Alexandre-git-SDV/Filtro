# syntax=docker/dockerfile:1

# ---- Build stage : compile the `filtro` binary ------------------------------
FROM rust:1-slim AS build

WORKDIR /src
# Only the Rust crate is needed to build
COPY Filtro_Rust/ ./Filtro_Rust/

WORKDIR /src/Filtro_Rust
RUN cargo build --release --locked

# ---- Runtime stage : minimal image with just the binary --------------------
FROM debian:stable-slim

# Run as a non-root user
RUN useradd --create-home --uid 10001 filtro
USER filtro
WORKDIR /work

COPY --from=build /src/Filtro_Rust/target/release/filtro /usr/local/bin/filtro

# The container *is* the CLI tool:
#   docker run --rm -v "$PWD:/work" filtro photo.jpg -f jaune -o out
ENTRYPOINT ["filtro"]
CMD ["--help"]
