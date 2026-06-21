#!/bin/sh
# =====================================================================
#  Set ready-to-use passwords for the demo accounts (one-shot, kanidm/server
#  image, runs AFTER provisioning created the persons).
#
#  Kanidm by design won't let you set a chosen password for a person over the
#  API (you'd use the browser credential-reset flow). For a turn-key local demo
#  we instead use `kanidmd recover-account`, which sets a fresh random password
#  and prints it — we capture those and write them to
#  /shared/demo-credentials.txt so the accounts are immediately usable.
#
#  Runs via the static busybox staged by `busybox-init`, sharing the kanidm_data
#  volume (recover-account talks to the running server over /data/kanidmd.sock).
# =====================================================================
set -eu
export PATH="/shared/bin:/sbin:/usr/sbin:/bin:/usr/bin"

CONFIG=/data/server.toml
SHARED=/shared
ADMIN_ACCOUNT="${KANIDM_ADMIN_ACCOUNT:-rehmann_admin}"
USER_ACCOUNT="${KANIDM_USER_ACCOUNT:-rehmann_user}"
OUT_FILE="$SHARED/demo-credentials.txt"

: > "$OUT_FILE"
echo "# Demo account credentials (Grand Casino Rehmann)."     >> "$OUT_FILE"
echo "# Log in at the frontend with these username/password pairs." >> "$OUT_FILE"
echo                                                          >> "$OUT_FILE"

set_pw() {
    acct="$1"
    role="$2"
    out="$(kanidmd recover-account "$acct" --config-path "$CONFIG" 2>&1)"
    pw="$(printf '%s' "$out" | sed -nE 's/.*new_password:[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
    if [ -z "$pw" ]; then
        echo "[demo-passwords] ERROR recovering $acct:" >&2
        echo "$out" >&2
        exit 1
    fi
    echo "$acct  ($role)" >> "$OUT_FILE"
    echo "  password: $pw" >> "$OUT_FILE"
    echo                   >> "$OUT_FILE"
    echo "[demo-passwords] $acct -> password set"
}

set_pw "$ADMIN_ACCOUNT" "admin role"
set_pw "$USER_ACCOUNT" "user role"

chmod 644 "$OUT_FILE"
echo "[demo-passwords] Credentials written to $OUT_FILE"
