FROM debian:trixie-slim

# Github release version
ARG GITHUB_RELEASE_VERSION

# Docker image arch
ARG TARGETARCH

# Setup working directory
WORKDIR /app

# Install necessary tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates poppler-utils && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Determine binary based on arch
RUN if [ "$TARGETARCH" = "amd64" ]; then \
    BINARY="quizler-x86_64-linux-gnu"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
    BINARY="quizler-aarch64-linux-gnu"; \
    else \
    echo "Unsupported architecture: $TARGETARCH" && exit 1; \
    fi && \
    # Download quizler binary
    curl -L -o quizler https://github.com/jacobtread/Quizler/releases/download/${GITHUB_RELEASE_VERSION}/$BINARY && \
    # Make binary executable
    chmod +x quizler

EXPOSE 80

CMD ["/app/quizler"]
