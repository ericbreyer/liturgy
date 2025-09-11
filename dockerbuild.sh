#!/bin/zsh
docker buildx build --platform linux/amd64 . -t ebreyer/lit
docker image ls
docker run -p 3000:3000 ebreyer/lit 