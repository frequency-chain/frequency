# FROM --platform=linux/amd64 ubuntu:20.04
FROM ubuntu:20.04
LABEL maintainer="Frequency"
LABEL description="Frequency CI base image"

WORKDIR /ci

RUN apt-get update && \
	apt-get install -y curl protobuf-compiler build-essential libclang-dev git file && \
	rm -rf /var/lib/apt/lists/*
