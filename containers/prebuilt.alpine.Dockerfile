FROM alpine:3.23

# Github release version
ARG GITHUB_RELEASE_VERSION

# Docker image arch
ARG TARGETARCH

# Setup working directory
WORKDIR /app

# Install necessary tools
RUN apk add --no-cache curl ca-certificates poppler-utils

# Determine binary based on arch
RUN if [ "$TARGETARCH" = "amd64" ]; then \
    BINARY="quizler-x86_64-linux-musl"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
    BINARY="quizler-aarch64-linux-musl"; \
    else \
    echo "Unsupported architecture: $TARGETARCH" && exit 1; \
    fi && \
    # Download quizler binary
    curl -L -o quizler https://github.com/jacobtread/Quizler/releases/download/${GITHUB_RELEASE_VERSION}/$BINARY && \
    # Make binary executable
    chmod +x quizler

EXPOSE 80

CMD ["/app/quizler"]
