#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: verifyCors.sh <identifier>"
    echo "Example: verifyCors.sh 41"
    exit 1
fi

IDENTIFIER="$1"
STARTED_BY_SCRIPT=false

is_running() {
    mgt ps --json 2>/dev/null | jq -e --arg id "$IDENTIFIER" \
        'map(select(.identifier == ($id + ".0"))) | length > 0' >/dev/null 2>&1
}

get_http_port() {
    mgt ps --json 2>/dev/null | jq -r --arg id "$IDENTIFIER" \
        'map(select(.identifier == ($id + ".0"))) | .[0].http'
}

if is_running; then
    echo "Container for $IDENTIFIER is already running."
else
    echo "Starting container for $IDENTIFIER..."
    mgt start "$IDENTIFIER"
    STARTED_BY_SCRIPT=true
    sleep 2
fi

PORT=$(get_http_port)
if [[ -z "$PORT" || "$PORT" == "null" ]]; then
    echo "ERROR: Could not determine HTTP port for $IDENTIFIER"
    exit 1
fi

BASE_URL="http://localhost:${PORT}/api/identity"
echo ""
echo "Testing CORS for $IDENTIFIER on $BASE_URL"
echo "==========================================="

echo ""
echo "--- GET with Origin header ---"
HEADERS=$(curl -s -o /dev/null -D - -H "Origin: http://localhost:3000" "$BASE_URL")
echo "$HEADERS" | grep -i "access-control" || echo "(no CORS headers found)"

echo ""
echo "--- OPTIONS preflight ---"
HEADERS=$(curl -s -o /dev/null -D - -X OPTIONS \
    -H "Origin: http://localhost:3000" \
    -H "Access-Control-Request-Method: GET" \
    "$BASE_URL")
echo "$HEADERS" | grep -i "access-control" || echo "(no CORS headers found)"

if [[ "$STARTED_BY_SCRIPT" == true ]]; then
    echo ""
    echo "Stopping container for $IDENTIFIER..."
    mgt stop "$IDENTIFIER"
fi

echo ""
echo "Done."
