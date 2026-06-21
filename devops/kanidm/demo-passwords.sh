#!/bin/sh
# Give the demo accounts ready-to-use passwords. Kanidm won't set a chosen
# password over the API, so we use `recover-account` (random password, printed)
# and write the results to /shared/demo-credentials.txt.
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
