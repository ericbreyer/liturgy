# 1) Frontend builder: build the Vue app
FROM node:20-bullseye-slim AS node-builder
WORKDIR /app/frontend
COPY liturgy-frontend/package*.json ./
RUN npm ci && \
    cp -a ../liturgy-frontend . || true
COPY liturgy-frontend .
# install deps, build, then remove sources and node_modules in the same layer to avoid leaving them in the image
RUN npm ci && npm run build

# 2) Rust builder: compile static musl binary
FROM rust:slim AS rust-builder
# musl + build deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools build-essential pkg-config libssl-dev ca-certificates && \
    rustup target add x86_64-unknown-linux-musl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
# copy Rust workspaces
COPY calendar_calc ./calendar_calc
COPY liturgy-backend ./liturgy-backend

# copy the frontend 'dist' into the Rust crate where server expects it
COPY --from=node-builder /app/frontend/dist ./liturgy-frontend/dist

# optionally set FRONTEND_DIR env if your build.rs reads it
ENV FRONTEND_DIR=../liturgy-frontend
# build static binary (adjust features/target as needed)
WORKDIR /usr/src/liturgy-backend
# build, move the resulting binary to a stable location (try known names), then remove sources in the same layer
RUN cargo build --release --target x86_64-unknown-linux-musl

# 3) Final runtime: small Alpine (musl) image
FROM alpine:3.18 AS runtime
# create non-root user
RUN adduser -D -u 10001 liturgy-backend
USER liturgy-backend
WORKDIR /app
COPY --from=rust-builder /usr/src/calendar_calc/calendar_data /app/calendar_calc/calendar_data
COPY --from=rust-builder /usr/src/liturgy-backend/target/x86_64-unknown-linux-musl/release/liturgy-backend /app/liturgy-backend
COPY --from=node-builder /app/frontend/dist /app/liturgy-frontend/dist

EXPOSE 3000
# ENTRYPOINT is the binary; CMD holds default args so users can override at docker run
ENTRYPOINT ["/app/liturgy-backend"]
CMD ["--host","0.0.0.0","--port","3000","--calendar-data-dir","/app/calendar_calc/calendar_data","--frontend-dir","/app/liturgy-frontend"]