#!/usr/bin/env bash
# =====================================================================
#  Staged bring-up for podman-compose.
#
#  podman-compose does NOT honour `depends_on: condition:
#  service_completed_successfully` for one-shot init containers (it maps
#  depends_on to podman's "requires", which expects the dependency to stay
#  *running* – our init containers exit, so dependents fail with "container
#  state improper"). docker compose handles the conditions natively, so with
#  Docker you can just run `docker compose up -d`.
#
#  This script starts the services in the correct order, waiting for each
#  one-shot to finish before starting what depends on it.
#
#  Usage:
#     ./podman-up.sh                      # full stack (docker-compose.yml)
#     ./podman-up.sh docker-compose.infra.yml   # infra only
# =====================================================================
set -euo pipefail
cd "$(dirname "$0")"

FILE="${1:-docker-compose.yml}"
PC=(podman-compose -f "$FILE")

wait_done() {
  local name="$1"
  echo ">> waiting for ${name} to complete..."
  podman wait "$name" >/dev/null
  local code
  code="$(podman inspect "$name" --format '{{.State.ExitCode}}')"
  if [ "$code" != "0" ]; then
    echo "!! ${name} exited with code ${code}:" >&2
    podman logs "$name" 2>&1 | tail -30 >&2
    exit 1
  fi
}

echo "== Postgres =="
"${PC[@]}" up -d --no-deps postgres

echo "== Kanidm: stage busybox =="
"${PC[@]}" up -d --no-deps busybox-init;     wait_done casino-busybox-init

echo "== Kanidm: TLS certificate =="
"${PC[@]}" up -d --no-deps kanidm-cert;      wait_done casino-kanidm-cert

echo "== Kanidm: server =="
"${PC[@]}" up -d --no-deps kanidm

echo "== Kanidm: recover idm_admin =="
"${PC[@]}" up -d --no-deps kanidm-recover;   wait_done casino-kanidm-recover

echo "== Kanidm: provision groups/users/oauth2 =="
"${PC[@]}" up -d --no-deps kanidm-provision; wait_done casino-kanidm-provision

echo "== Kanidm: set demo account passwords =="
"${PC[@]}" up -d --no-deps kanidm-demo-passwords; wait_done casino-kanidm-demo-passwords

# Backend + frontend only exist in the combined compose file.
if grep -qE '^[[:space:]]*frontend:' "$FILE"; then
  echo "== Building backend + frontend images =="
  "${PC[@]}" build backend frontend
  echo "== Backend =="
  "${PC[@]}" up -d --no-deps backend
  echo "== Frontend =="
  "${PC[@]}" up -d --no-deps frontend
fi

echo
echo "Stack is up."
echo "Demo account password reset links: ./kanidm/secrets/reset-links.txt"
