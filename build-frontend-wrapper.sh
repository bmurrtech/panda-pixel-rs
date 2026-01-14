#!/bin/bash
# Cross-platform wrapper - on Unix, just run the script directly
# This file is for Unix systems; Windows will use build-frontend-wrapper.cmd
exec "$(dirname "$0")/build-frontend.sh" "$@"
