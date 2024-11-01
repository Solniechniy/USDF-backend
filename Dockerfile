# # Use a Rust base image with Cargo installed
# FROM rust:1.78.0 AS builder

# # Set the working directory inside the container
# WORKDIR /usr/src/app

# # Copy the Cargo.toml and Cargo.lock files
# COPY Cargo.toml Cargo.lock ./

# # Create an empty src directory to trick Cargo into thinking it's a valid Rust project
# RUN mkdir src && echo "fn main() {}" > src/main.rs

# # Build the dependencies without the actual source code to cache dependencies separately
# RUN cargo build --release

# # Now copy the source code
# COPY ./src ./src

# # Build your application
# RUN cargo build --release

# # Start a new stage to create a smaller image without unnecessary build dependencies
# FROM debian:bookworm-slim

# # Set the working directory
# WORKDIR /usr/src/app

# # Copy the built binary from the previous stage
# COPY --from=builder /usr/src/app/target/release/usdf-back /bin/usdf-back

# # Exposing port
# EXPOSE 8080

# # Command to run the application
# CMD ["usdf-back"]

# Builder image
FROM rust:1.78.0 AS builder

# We want to build against musl target to deploy in a `scratch` (without glibc) later
ENV TARGET x86_64-unknown-linux-musl

# Setup musl target and it's dependencies
RUN rustup target add "$TARGET" && apt-get update && apt-get install -y musl-tools

# Prefetch dependencies
COPY Cargo.lock Cargo.toml ./
RUN cargo fetch

# Build from source code
COPY . .
RUN cargo build --release --locked --target "$TARGET" && cp "/target/${TARGET}/release/usdf-back" ./

# Runtime image
FROM scratch AS runtime

# Exposing port
EXPOSE 8080

# Copy the built binary
COPY --from=builder /usdf-back /bin/usdf-back

# Make it the default command
ENTRYPOINT ["/bin/usdf-back"]
