#!/bin/sh
set -e

if [ "$USE_UNIX_SOCKET" = "true" ]; then
	export BACKEND_UPSTREAM="unix//tmp/push-platform.sock"
else
	export BACKEND_UPSTREAM="localhost:${BACKEND_PORT:-8080}"
fi

if [ -n "$SITE_NAME" ]; then
	export SITE_ADDRESS="$SITE_NAME"
else
	export SITE_ADDRESS=":80"
fi

exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
