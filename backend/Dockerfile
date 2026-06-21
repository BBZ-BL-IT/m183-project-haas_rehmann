# syntax=docker/dockerfile:1

# =====================================================================
#  Build stage – compile the Axum backend in release mode.
# ---------------------------------------------------------------------
#  We do NOT use sqlx's compile-time `query!` macros, so NO database is
#  needed at build time (all queries are runtime-checked – see db.rs).
#  The `migrate!` macro embeds the ./migrations SQL into the binary, so
#  the migrations directory must be present here (it is, via COPY .).
# =====================================================================
FROM rust:1-bookworm AS builder

WORKDIR /app

# System dependencies for the TLS stack (rustls + aws-lc-rs needs cmake and a
# C/clang toolchain) and general native builds.
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    clang \
    libclang-dev \
    pkg-config \
    perl \
    make \
    && rm -rf /var/lib/apt/lists/*

# Prime the dependency cache: build a stub crate with only the manifests so the
# (expensive) dependency compilation is cached as long as Cargo.toml/lock are
# unchanged. The real sources are copied afterwards.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release --locked && \
    rm -rf src

# Now copy the real sources + migrations and build for real.
COPY src ./src
COPY migrations ./migrations
# `touch` so cargo's mtime-based fingerprint sees the real sources as newer than
# the stub build (COPY preserves the host mtimes, which would otherwise be older
# and make cargo skip recompiling – shipping the empty stub binary).
RUN touch src/main.rs && \
    cargo build --release --locked && \
    cp target/release/m183-backend /m183-backend

# =====================================================================
#  Runtime stage – minimal Debian with just CA certificates (needed for
#  the outbound HTTPS calls to Kanidm's token endpoint).
# =====================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user.
RUN useradd --system --create-home --uid 10001 appuser
USER appuser

WORKDIR /app
COPY --from=builder /m183-backend /usr/local/bin/m183-backend

ENV SERVER_PORT=8080
EXPOSE 8080

CMD ["m183-backend"]
