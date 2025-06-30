# Use the official Rust image as base
FROM rust:1.75 as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Use a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/solana-http-server .

# Expose the port
EXPOSE 3000

# Run the application
CMD ["./solana-http-server"]