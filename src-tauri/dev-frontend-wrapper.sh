#!/bin/bash
# Wrapper that finds the dev script relative to this file's location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.." && ./dev-frontend.sh
