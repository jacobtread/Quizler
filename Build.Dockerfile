# Dockerfile that manually builds the entire app

# Frontend build stage
# =====================
FROM node:lts-alpine as frontend

# Frontend building
WORKDIR /app

# Copy the package json and lock file
COPY frontend/package.json ./
COPY frontend/yarn.lock ./

# Install the dependencies
RUN yarn install

# Copy the remaining source
COPY frontend ./

# Build the app
RUN yarn build

# Backend build stage
# =====================
FROM rust:1.69.0 as backend

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /app

# Copy cargo project files
COPY ./backend/Cargo.toml .
COPY ./backend/Cargo.lock .

# Create dummy contents for main source
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs

# Cargo build the dummy project for dependency caching
RUN cargo build --target x86_64-unknown-linux-musl --release

# Remove dummy src 
RUN rm -rf ./src

# Copy real source code over
COPY ./backend/src ./src

# Copy the frontend over
COPY --from=frontend /app/dist ./public


# Update the modified time on the project files so they recompile
RUN touch -a -m ./src/main.rs

# Cargo build real source code
RUN cargo build --target x86_64-unknown-linux-musl --release



# Run stage
# =====================
FROM alpine:latest

# Copy our build
COPY --from=backend /app/target/x86_64-unknown-linux-musl/release/quizler ./

ENV QUIZLER_PORT=80
ENV RUST_LOG="quizler=info"

EXPOSE 80

CMD ["/quizler"]