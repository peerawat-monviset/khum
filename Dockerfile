# Build stage
FROM rust:slim AS builder

WORKDIR /app
COPY . .

# Compile release binary with host-specific CPU instructions
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

# Runner stage
FROM debian:bookworm-slim AS runner
WORKDIR /app

# Copy executable and static public assets
COPY --from=builder /app/target/release/khum /app/
COPY --from=builder /app/public /app/public

# Expose port
ENV PORT=8000
EXPOSE 8000

CMD ["/app/khum"]
