# Use the official Rust image as the base image
FROM rust:1.72 as builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage with Ubuntu 22.04
FROM ubuntu:22.04

# Install OpenSSL - often needed for Rust programs
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/app/target/release/geommdb /usr/local/bin/

# Set the startup command to run your binary
CMD ["geommdb"]