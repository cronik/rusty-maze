FROM rust:1.57 as build

WORKDIR /usr/src/rusty_maze
COPY Cargo.lock Cargo.toml ./
COPY src/ ./src
RUN cargo install --path .

FROM debian:stable-slim
# `rest -w` is a workaround to for this issue https://github.com/moby/moby/issues/33794
RUN echo '#!/usr/bin/env sh\nreset -w && maze "$@"' > /usr/bin/entrypoint.sh \
    && chmod 755 /usr/bin/entrypoint.sh
COPY --from=build /usr/local/cargo/bin/rusty_maze /usr/bin/maze
ENTRYPOINT ["/usr/bin/entrypoint.sh"]