#!/bin/sh
# =====================================================================
#  Kanidm idm_admin recovery (one-shot, kanidm/server image, runs AFTER the
#  server is up). In Kanidm 1.10 `recover-account` is an ONLINE operation: it
#  talks to the running server over its admin unix socket (/data/kanidmd.sock),
#  so this shares the kanidm_data volume with the server.
#
#  The freshly recovered password is written to /shared/idm_admin.password for
#  the provisioning step. We always re-recover so the saved password stays in
#  sync with the current database (the host secrets dir survives `down -v`).
# =====================================================================
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
