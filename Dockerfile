# Dockerfile that uses a precompiled version
FROM alpine:latest

# Update system and install curl
RUN apk update && apk upgrade
RUN apk add curl

# Download release
RUN curl -LJ -o quizler https://github.com/jacobtread/Quizler/releases/download/v0.1.0/quizler-linux

# Make the server executable
RUN chmod +x ./quizler

EXPOSE 80

CMD ["/quizler"]