FROM ubuntu:22.04 AS image
# Run `make dist` before this command or build the artifact in CI.
COPY dist/bin/malachitebft-eth-app /usr/local/bin/malachitebft-eth-app

RUN chmod +x /usr/local/bin/malachitebft-eth-app

# Copy licenses
COPY LICENSE ./

EXPOSE 30303 30303/udp 9001 8551 8545 8546
ENTRYPOINT ["/usr/local/bin/malachitebft-eth-app"]
