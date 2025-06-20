FROM rust:1.85-slim-bookworm AS build

RUN apt-get update && apt-get install -y make && rustup component add clippy rustfmt

FROM build as claude

# Install necessary packages
RUN apt-get update -y && apt-get install -y npm gh

# Install Claude CLI
RUN npm install -g @anthropic-ai/claude-code
