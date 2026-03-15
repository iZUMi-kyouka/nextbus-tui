#!/usr/bin/env bash
# Configure git to use the project-tracked hooks in .githooks/
set -euo pipefail

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/pre-push

echo "✓ Git hooks installed (core.hooksPath = .githooks)"
