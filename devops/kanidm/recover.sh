#!/bin/sh
# Recover idm_admin (runs after the server is up; recover-account is online via
# /data/kanidmd.sock) and write the password to /shared/idm_admin.password.
# Always re-recovers so the saved password matches the current database.
set -eu
export PATH="/shared/bin:/sbin:/usr/sbin:/bin:/usr/bin"

CONFIG=/data/server.toml
SHARED=/shared
SOCKET=/data/kanidmd.sock

echo "[recover] Waiting for the Kanidm admin socket ($SOCKET)..."
i=0
until [ -S "$SOCKET" ]; do
    i=$((i + 1))
    if [ "$i" -ge 60 ]; then
        echo "[recover] ERROR: admin socket never appeared." >&2
        exit 1
    fi
    sleep 2
done

echo "[recover] Recovering idm_admin..."
OUT="$(kanidmd recover-account idm_admin --config-path "$CONFIG" 2>&1)"

# Output line looks like:  new_password: "xxxxxxxx"
PW="$(printf '%s' "$OUT" | sed -nE 's/.*new_password:[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
if [ -z "$PW" ]; then
    echo "[recover] ERROR: could not parse the recovered password from:" >&2
    echo "$OUT" >&2
    exit 1
fi

printf '%s' "$PW" > "$SHARED/idm_admin.password"
chmod 600 "$SHARED/idm_admin.password"
echo "[recover] idm_admin password saved to $SHARED/idm_admin.password"
