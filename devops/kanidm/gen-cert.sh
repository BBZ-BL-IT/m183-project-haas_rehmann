#!/bin/sh
# Generate a self-signed CA + leaf cert valid for both names Kanidm is reached
# by: "localhost" (browser) and "kanidm" (backend over the compose network).
# kanidmd cert-generate only covers one name, hence openssl.
set -eu

D=/data
S=/shared

if [ -f "$D/chain.pem" ] && [ -f "$D/key.pem" ] && [ -f "$D/ca.pem" ]; then
    echo "[cert] certificate already present, skipping generation."
else
    echo "[cert] generating CA + leaf certificate (SAN: localhost, kanidm)..."
    openssl req -x509 -newkey rsa:2048 -nodes \
        -keyout "$D/ca-key.pem" -out "$D/ca.pem" -days 3650 \
        -subj "/CN=Grand Casino Dev CA/O=GrandCasino"
    openssl req -newkey rsa:2048 -nodes \
        -keyout "$D/key.pem" -out "$D/leaf.csr" \
        -subj "/CN=localhost/O=GrandCasino"
    cat > "$D/san.ext" <<EOF
subjectAltName=DNS:localhost,DNS:kanidm,IP:127.0.0.1
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
EOF
    openssl x509 -req -in "$D/leaf.csr" \
        -CA "$D/ca.pem" -CAkey "$D/ca-key.pem" -CAcreateserial \
        -out "$D/cert.pem" -days 3650 -extfile "$D/san.ext"
    # Kanidm wants the full chain (leaf + CA) in tls_chain.
    cat "$D/cert.pem" "$D/ca.pem" > "$D/chain.pem"
fi

cp "$D/ca.pem" "$S/kanidm-ca.pem"
echo "[cert] done. CA exported to $S/kanidm-ca.pem"
