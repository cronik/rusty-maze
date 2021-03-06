FROM rust:1.50 as build

WORKDIR /usr/src/rusty_maze
COPY Cargo.lock Cargo.toml ./
COPY src/ ./src
RUN cargo install --path .

FROM debian:stable-slim
COPY --from=build /usr/local/cargo/bin/rusty_maze /usr/bin/maze
# `rest -w` is a workaround to for this issue https://github.com/moby/moby/issues/33794
ENTRYPOINT ["sh", "-c", "reset -w && maze"]