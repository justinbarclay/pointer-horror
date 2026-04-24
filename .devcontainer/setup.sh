#!/usr/bin/env bash
# .devcontainer/setup.sh
# Runs once after container creation (postCreateCommand).
# Sets up all tools not provided by the base image or devcontainer features.

set -euo pipefail

# ── bun ──────────────────────────────────────────────────────────────────────
echo "→ Installing bun..."
curl -fsSL https://bun.sh/install | bash

BUN="$HOME/.bun/bin/bun"

# Persist bun on PATH for all future shells in the container.
PROFILE="$HOME/.bashrc"
if ! grep -q '\.bun/bin' "$PROFILE"; then
  echo 'export PATH="$HOME/.bun/bin:$PATH"' >> "$PROFILE"
fi

# ── TypeScript dependencies ───────────────────────────────────────────────────
echo "→ Installing TypeScript dependencies..."
"$BUN" install --cwd typescript

# ── Rust launcher (pre-build so the first run is fast) ───────────────────────
echo "→ Pre-building launcher..."
cargo build --manifest-path rust/Cargo.toml --bin launcher

echo ""
echo "✓ Setup complete."
