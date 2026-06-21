#!/bin/sh
# =====================================================================
#  Generate Kanidm's TLS certificate (one-shot, alpine/openssl image, runs
#  BEFORE the server). Produces a self-signed CA + a leaf certificate valid for
#  BOTH names Kanidm is reached by:
#    - localhost : what the BROWSER uses (https://localhost:8443)
#    - kanidm    : what the BACKEND container uses over the compose network
#  This is why we generate the cert ourselves instead of `kanidmd cert-generate`
#  (which only covers a single name). No /etc/hosts entry needed.
#
#  Files written to the kanidm_data volume (/data) + CA copied to /shared so the
#  backend can trust it via OIDC_CA_CERT.
# =====================================================================
set -eu

D=/data
S=/shared

if [ -f "$D/chain.pem" ] && [ -f "$D/key.pem" ] && [ -f "$D/ca.pem" ]; then
    echo "[cert] certificate already present, skipping generation."
else
    echo "[cert] generating CA + leaf certificate (SAN: localhost, kanidm)..."

    # 1. Self-signed CA.
    openssl req -x509 -newkey rsa:2048 -nodes \
        -keyout "$D/ca-key.pem" -out "$D/ca.pem" -days 3650 \
        -subj "/CN=Grand Casino Dev CA/O=GrandCasino"

    # 2. Leaf key + CSR.
    openssl req -newkey rsa:2048 -nodes \
        -keyout "$D/key.pem" -out "$D/leaf.csr" \
        -subj "/CN=localhost/O=GrandCasino"

    # 3. Sign the leaf with both SANs.
    cat > "$D/san.ext" <<EOF
subjectAltName=DNS:localhost,DNS:kanidm,IP:127.0.0.1
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
EOF
    openssl x509 -req -in "$D/leaf.csr" \
        -CA "$D/ca.pem" -CAkey "$D/ca-key.pem" -CAcreateserial \
        -out "$D/cert.pem" -days 3650 -extfile "$D/san.ext"

    # 4. Kanidm wants the full chain (leaf + CA) in tls_chain.
    cat "$D/cert.pem" "$D/ca.pem" > "$D/chain.pem"
fi

# Export the CA so the backend can trust Kanidm over HTTPS.
cp "$D/ca.pem" "$S/kanidm-ca.pem"
echo "[cert] done. CA exported to $S/kanidm-ca.pem"
