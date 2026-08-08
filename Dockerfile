FROM rust:1-alpine3.21 as builder

SHELL ["/bin/ash", "-o", "pipefail", "-c"]

RUN apk update && \
  apk add --no-cache bash curl npm libc-dev binaryen

RUN npm install -g sass@1.77.8

RUN curl --proto '=https' --tlsv1.3 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.7/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM rust:1-alpine3.21 as runner

RUN addgroup -S app && adduser -S -G app app

WORKDIR /app

COPY --from=builder /work/target/release/server /app/
COPY --from=builder /work/target/site /app/site

RUN chown -R app:app /app

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

USER app

CMD ["/app/server"]
