#!/bin/bash
# Setup git hooks for bittensor-rs

REPO_ROOT="$(git rev-parse --show-toplevel)"
git config core.hooksPath "$REPO_ROOT/.githooks"

echo "Git hooks configured. Pre-commit will format code, pre-push will run CI checks."
