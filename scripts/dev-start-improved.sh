#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/dev-start-managed.sh" start-backend
"$SCRIPT_DIR/dev-start-managed.sh" start-frontend
"$SCRIPT_DIR/dev-start-managed.sh" status
