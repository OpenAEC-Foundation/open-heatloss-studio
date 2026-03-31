# =============================================================================
# Stage 1: Build Rust API binary
# =============================================================================
FROM rust:1-bookworm AS rust-builder

WORKDIR /build

# Copy workspace and crate manifests first (Docker layer caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/isso51-core/Cargo.toml crates/isso51-core/Cargo.toml
COPY crates/isso51-ifcx/Cargo.toml crates/isso51-ifcx/Cargo.toml
COPY crates/isso51-api/Cargo.toml crates/isso51-api/Cargo.toml

# Remove src-tauri from workspace members (Tauri deps don't build here)
RUN sed -i '/"src-tauri"/d' Cargo.toml

# Create dummy source files so cargo can resolve deps
RUN mkdir -p crates/isso51-core/src && echo "pub fn dummy() {}" > crates/isso51-core/src/lib.rs \
 && mkdir -p crates/isso51-ifcx/src && echo "pub fn dummy() {}" > crates/isso51-ifcx/src/lib.rs \
 && mkdir -p crates/isso51-api/src && echo "fn main() {}" > crates/isso51-api/src/main.rs

# Pre-build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release -p isso51-api 2>/dev/null || true

# Cache-bust: changes when source code changes (git commit hash)
ARG SOURCE_HASH
# Copy actual source code
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

# Cache-bust: changes when source code changes (git commit hash)
ARG SOURCE_HASH
# Copy frontend source and schemas (needed for type generation)
COPY frontend/ .
COPY schemas/ /build/schemas/

# OIDC config passed at build time for Vite
ARG VITE_OIDC_ISSUER
ARG VITE_OIDC_CLIENT_ID
ARG VITE_APP_VERSION
ENV VITE_OIDC_ISSUER=${VITE_OIDC_ISSUER}
ENV VITE_OIDC_CLIENT_ID=${VITE_OIDC_CLIENT_ID}
ENV VITE_APP_VERSION=${VITE_APP_VERSION}

# Write version to .env.local so Vite picks it up reliably (overrides .env files)
RUN echo "VITE_APP_VERSION=${VITE_APP_VERSION}" >> .env.local

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

# Install Python + ifc-tool for server-side IFC import
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

COPY tools/ifc-tool /tmp/ifc-tool-src
RUN python3 -m venv /opt/ifc-tool-venv \
 && /opt/ifc-tool-venv/bin/pip install --no-cache-dir /tmp/ifc-tool-src \
 && rm -rf /tmp/ifc-tool-src

# Create data directory for SQLite
RUN mkdir -p /data && chown app:app /data \
 && chown -R app:app /opt/ifc-tool-venv
VOLUME /data

USER app

ENV PORT=3001
ENV DATABASE_URL=sqlite:///data/isso51.db?mode=rwc
ENV STATIC_DIR=/app/static
EXPOSE 3001

ENTRYPOINT ["/app/isso51-api"]
