# =============================================================================
# Stage 1: Build Rust API binary
# =============================================================================
FROM rust:1-bookworm AS rust-builder

WORKDIR /build

# Copy workspace and crate manifests first (Docker layer caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/isso51-core/Cargo.toml crates/isso51-core/Cargo.toml
COPY crates/isso51-api/Cargo.toml crates/isso51-api/Cargo.toml

# Remove src-tauri from workspace members (Tauri deps don't build here)
RUN sed -i '/"src-tauri"/d' Cargo.toml

# Create dummy source files so cargo can resolve deps
RUN mkdir -p crates/isso51-core/src && echo "pub fn dummy() {}" > crates/isso51-core/src/lib.rs \
 && mkdir -p crates/isso51-api/src && echo "fn main() {}" > crates/isso51-api/src/main.rs

# Pre-build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release -p isso51-api 2>/dev/null || true

# Copy actual source code (invalidates cache — triggers real build below)
COPY crates/ crates/
COPY schemas/ schemas/

# Ensure src-tauri is still excluded after full copy
RUN sed -i '/"src-tauri"/d' Cargo.toml

# Touch source files to bust the cargo cache from the dummy build
RUN find crates/ -name '*.rs' -exec touch {} +

# Build the real binary
RUN cargo build --release -p isso51-api

# =============================================================================
# Stage 2: Build frontend
# =============================================================================
FROM node:22-bookworm-slim AS node-builder

WORKDIR /build/frontend

# Copy package files first (layer caching)
COPY frontend/package.json frontend/package-lock.json* ./

RUN npm ci

# Copy frontend source and schemas (needed for type generation)
COPY frontend/ .
COPY schemas/ /build/schemas/

# Build for production
RUN npm run build

# =============================================================================
# Stage 3: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /usr/sbin/nologin app

WORKDIR /app

# Copy binary from Rust builder
COPY --from=rust-builder /build/target/release/isso51-api /app/isso51-api

# Copy frontend dist from Node builder
COPY --from=node-builder /build/frontend/dist /app/static

# Create data directory for SQLite
RUN mkdir -p /data && chown app:app /data

USER app

ENV PORT=3001
ENV STATIC_DIR=/app/static
EXPOSE 3001

ENTRYPOINT ["/app/isso51-api"]
