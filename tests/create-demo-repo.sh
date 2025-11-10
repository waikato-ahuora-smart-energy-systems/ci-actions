#!/usr/bin/env bash
set -e

# Allow custom repo name as first argument, default to "demo-repo"
REPO_DIR="${1:-dummy-repo}"

# Create and enter the repo directory
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"

# Initialize git
git init

# Configure user (local only)
git config user.name "Demo User"
git config user.email "demo@example.com"

# Create first commit
echo "Hello World" > file.txt
git add file.txt
git commit -m "Initial commit"

# Second commit
echo "Second line" >> file.txt
git add file.txt
git commit -m "Add second line"

# Tag first version
git tag v0.1

# Third commit
echo "Third line" >> file.txt
git add file.txt
git commit -m "Add third line"

# Fourth commit
echo "Fourth line" >> file.txt
git add file.txt
git commit -m "Add fourth line"

# Tag another version
git tag -a v1.0 -m "Release version 1.0"

# Show results
echo
echo "== Commits =="
git log --oneline --decorate

echo
echo "== Tags =="
git tag -n