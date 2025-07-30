# Stage 1: Multi-platform build environment (using buildx for cross-architecture compilation)
FROM --platform=$BUILDPLATFORM rustlang/rust:nightly AS builder

# Install cross-compilation dependencies and protoc
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Clone project code
RUN git clone https://github.com/MetisProtocol/malaketh-layered.git .

# Set Rust target based on architecture
ARG TARGETARCH
RUN case ${TARGETARCH} in \
        amd64) TARGET=x86_64-unknown-linux-gnu ;; \
        arm64) TARGET=aarch64-unknown-linux-gnu ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    rustup target add ${TARGET}

# Cross-compile the project (specify target architecture)
ARG TARGETARCH
RUN case ${TARGETARCH} in \
        amd64) TARGET=x86_64-unknown-linux-gnu ;; \
        arm64) TARGET=aarch64-unknown-linux-gnu ;; \
    esac && \
    cargo build --release --target ${TARGET}

# Stage 2: Multi-platform runtime environment
FROM --platform=$TARGETPLATFORM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy compiled binaries based on target architecture
ARG TARGETARCH
COPY --from=builder /app/target/$(case ${TARGETARCH} in \
    amd64) echo "x86_64-unknown-linux-gnu" ;; \
    arm64) echo "aarch64-unknown-linux-gnu" ;; \
esac)/release/malachitebft-eth-app /app/
COPY --from=builder /app/target/$(case ${TARGETARCH} in \
    amd64) echo "x86_64-unknown-linux-gnu" ;; \
    arm64) echo "aarch64-unknown-linux-gnu" ;; \
esac)/release/malachitebft-eth-utils /app/

# Add application directory to system path
ENV PATH="/app:${PATH}"

# Expose required ports
EXPOSE 8545 8551 9090 3000

# Default command to run the application
CMD ["malachitebft-eth-app"]
    
