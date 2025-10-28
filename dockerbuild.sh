#!/bin/zsh
clear

echo
echo "----------------------------------------"
echo "| 1. Cleaning previous build artifacts |"
echo "----------------------------------------"
echo

cargo clean

echo
echo "------------------------------"
echo "| 2. Building backend binary |"
echo "------------------------------"
echo

docker buildx build --platform linux/amd64 . -t ebreyer/lit

echo
echo "---------------------------"
echo "| 3. Pushing Docker image |"
echo "---------------------------"
echo

docker push ebreyer/lit:latest

echo
echo "-----------------------------"
echo "| 4. Triggering Render.com  |"
echo "| deployment for backend    |"
echo "-----------------------------"
echo

curl https://api.render.com/deploy/srv-d3vd3k6uk2gs73eg1kf0\?key\=EGwAysNO1Q0