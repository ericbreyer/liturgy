#!/bin/zsh
docker buildx build --platform linux/amd64 . -t ebreyer/lit
docker image ls
docker run -p 80:80 ebreyer/lit 