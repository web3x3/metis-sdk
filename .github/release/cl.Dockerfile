# Use Rust Nightly version base image
FROM rustlang/rust:nightly

# Set working directory
WORKDIR /app

# Install necessary dependencies, including protoc
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Clone project code
RUN git clone https://github.com/MetisProtocol/malaketh-layered.git .

# Build the project
RUN cargo build --release

# Copy the generated binary to a new lightweight image
#FROM debian:bullseye-slim
#FROM debian:bookworm-slim
#FROM ubuntu:22.04
FROM ubuntu:24.04

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the generated binary from the previous build image
COPY --from=0 /app/target/release/malachitebft-eth-app /app/
#COPY --from=0 /app/target/release/malachitebft-eth-cli /app/
#COPY --from=0 /app/target/release/malachitebft-eth-engine /app/
COPY --from=0 /app/target/release/malachitebft-eth-utils /app/

# Set environment variables
ENV PATH="/app:${PATH}"

# Expose necessary ports
EXPOSE 8545 8551 9090 3000

# Run the application (assuming you want to run malachitebft-eth-app here)
CMD ["malachitebft-eth-app"]

