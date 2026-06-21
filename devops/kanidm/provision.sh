#!/bin/sh
# =====================================================================
#  Kanidm provisioning (one-shot, kanidm/tools image, runs AFTER idm_admin has
#  been recovered). Logs in as idm_admin and declaratively creates:
#
#    - groups:   casino_admins, casino_users
#    - persons:  $KANIDM_ADMIN_ACCOUNT (admin), $KANIDM_USER_ACCOUNT (user)
#    - an OAuth2 confidential client "m183-backend" with:
#        * redirect URLs for the combined (8081) and direct (8080) setups
#        * scope maps (openid profile email groups)
#        * a "roles" claim map: casino_admins -> admin, casino_users -> user
#
#  Outputs written to /shared (host ./kanidm/secrets):
#    - oauth2_client_secret  : the backend's OIDC_CLIENT_SECRET
#    - reset-links.txt       : credential-reset URLs to set the demo passwords
#
#  Runs via the static busybox staged by `busybox-init`. All "create" steps
#  tolerate "already exists" so re-running is safe.
# =====================================================================
set -eu
export PATH="/shared/bin:/sbin:/usr/sbin:/bin:/usr/bin"

SHARED=/shared
CLIENT_ID="${OIDC_CLIENT_ID:-m183-backend}"
ADMIN_ACCOUNT="${KANIDM_ADMIN_ACCOUNT:-rehmann_admin}"
USER_ACCOUNT="${KANIDM_USER_ACCOUNT:-rehmann_user}"

# Redirect URLs we register (combined stack via frontend proxy + direct backend).
REDIRECT_PRIMARY="${OIDC_REDIRECT_URI:-http://localhost:8081/auth/callback}"
REDIRECT_SECONDARY="http://localhost:8080/auth/callback"

# --- kanidm CLI auth via env (dev: accept the self-signed certificate) --------
export KANIDM_URL="${KANIDM_URL:-https://kanidm:8443}"
export KANIDM_NAME=idm_admin
export KANIDM_ACCEPT_INVALID_CERTS=true
export HOME=/tmp
mkdir -p "$HOME" 2>/dev/null || true

if [ ! -s "$SHARED/idm_admin.password" ]; then
    echo "[provision] ERROR: $SHARED/idm_admin.password missing (recover step did not run?)." >&2
    exit 1
fi
KANIDM_PASSWORD="$(cat "$SHARED/idm_admin.password")"
export KANIDM_PASSWORD

# --- Wait for the server, then authenticate -----------------------------------
echo "[provision] Logging in to $KANIDM_URL as idm_admin..."
i=0
until kanidm login >/dev/null 2>&1; do
    i=$((i + 1))
    if [ "$i" -ge 60 ]; then
        echo "[provision] ERROR: could not log in to Kanidm after 60 tries." >&2
        exit 1
    fi
    sleep 3
done
echo "[provision] Logged in."

# Helper: tolerate "already exists" so provisioning is idempotent.
try() {
    if "$@"; then
        return 0
    fi
    echo "[provision] (ignored non-fatal failure: $*)"
    return 0
}

echo "[provision] Creating groups..."
try kanidm group create casino_admins
try kanidm group create casino_users

echo "[provision] Creating demo persons..."
try kanidm person create "$ADMIN_ACCOUNT" "Casino Admin"
try kanidm person update "$ADMIN_ACCOUNT" --mail "$ADMIN_ACCOUNT@kanidm"
try kanidm person create "$USER_ACCOUNT" "Casino User"
try kanidm person update "$USER_ACCOUNT" --mail "$USER_ACCOUNT@kanidm"

echo "[provision] Assigning group memberships..."
try kanidm group add-members casino_admins "$ADMIN_ACCOUNT"
try kanidm group add-members casino_users "$USER_ACCOUNT"
# Admins are users too (so the admin also gets the "user" role).
try kanidm group add-members casino_users "$ADMIN_ACCOUNT"

echo "[provision] Creating OAuth2 client '$CLIENT_ID'..."
# (Plain-http redirects to localhost are accepted by default in current Kanidm.)
try kanidm system oauth2 create "$CLIENT_ID" "Grand Casino Rehmann" "http://localhost:8081"
try kanidm system oauth2 add-redirect-url "$CLIENT_ID" "$REDIRECT_PRIMARY"
try kanidm system oauth2 add-redirect-url "$CLIENT_ID" "$REDIRECT_SECONDARY"

echo "[provision] Configuring scope maps..."
try kanidm system oauth2 update-scope-map "$CLIENT_ID" casino_users openid profile email groups
try kanidm system oauth2 update-scope-map "$CLIENT_ID" casino_admins openid profile email groups

echo "[provision] Configuring 'roles' claim map (group -> role)..."
try kanidm system oauth2 update-claim-map "$CLIENT_ID" roles casino_admins admin
try kanidm system oauth2 update-claim-map "$CLIENT_ID" roles casino_users user
try kanidm system oauth2 update-claim-map-join "$CLIENT_ID" roles array

echo "[provision] Exporting OAuth2 client secret..."
SECRET="$(kanidm system oauth2 show-basic-secret "$CLIENT_ID" 2>/dev/null | tr -d '[:space:]')"
if [ -n "$SECRET" ]; then
    printf '%s' "$SECRET" > "$SHARED/oauth2_client_secret"
    # World-readable: the backend container reads this as a non-root user via a
    # read-only bind mount (local dev secret, the directory is git-ignored).
    chmod 644 "$SHARED/oauth2_client_secret"
    echo "[provision] Client secret written to $SHARED/oauth2_client_secret"
else
    echo "[provision] WARNING: could not read the client secret automatically." >&2
fi

# --- Credential reset links for the demo accounts -----------------------------
# Kanidm (by design) does not let us set a raw password for a person over the
# API; instead we mint a reset token. Open the printed URL to set the password.
echo "[provision] Generating credential-reset links..."
{
    echo "# Open these URLs in the browser to set the demo account passwords."
    echo "# (Accept the self-signed certificate warning for https://kanidm:8443)"
    echo
    echo "## Admin account ($ADMIN_ACCOUNT) -> role admin"
    kanidm person credential create-reset-token "$ADMIN_ACCOUNT" --ttl 86400 2>/dev/null || true
    echo
    echo "## User account ($USER_ACCOUNT) -> role user"
    kanidm person credential create-reset-token "$USER_ACCOUNT" --ttl 86400 2>/dev/null || true
} | tee "$SHARED/reset-links.txt"

echo "[provision] Done. See $SHARED/reset-links.txt for the password reset links."
