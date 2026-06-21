# DevOps – Grand Casino Rehmann

Local container setup for the whole stack: **PostgreSQL**, **Kanidm** (OIDC
identity provider), the **Axum backend**, and the **Vue frontend**.

```
devops/
├── .env.example                 # copy to .env and edit
├── docker-compose.yml           # ⭐ everything (postgres + kanidm + backend + frontend)
├── docker-compose.infra.yml     # postgres + kanidm only
├── docker-compose.backend.yml   # backend only (built locally / or pulled)
├── docker-compose.frontend.yml  # frontend only (built locally / or pulled)
├── podman-up.sh                 # staged bring-up for podman (see below)
└── kanidm/
    ├── server.toml              # kanidm server config
    ├── cert.sh                  # one-shot: generate TLS cert (pre-server)
    ├── recover.sh               # one-shot: recover idm_admin (post-server)
    ├── provision.sh             # one-shot: groups / users / oauth2 client
    └── secrets/                 # generated at runtime (git-ignored)
```

> This setup was validated end-to-end with **rootless podman** and Kanidm
> **1.10.3**. It also works with Docker (`docker compose ...`).

## 1. Prerequisites

- A container engine. Both are in the project `flake.nix`:
  - **podman** + **podman-compose** (rootless, no daemon — recommended here), or
  - **docker** + the compose plugin (the Docker daemon must be running).
- Add the Kanidm hostname to your hosts file so the **browser** can reach it at
  the same URL the backend uses:

  ```sh
  echo "127.0.0.1 kanidm" | sudo tee -a /etc/hosts
  ```

- Configuration:

  ```sh
  cd devops
  cp .env.example .env
  # edit .env: set a strong PRIVATE_COOKIE_KEY  (openssl rand -base64 64)
  # Images are built locally by default — no Docker Hub account needed.
  ```

- Rootless podman needs a `policy.json` and `registries.conf` (usually already
  present on NixOS via `virtualisation.containers.enable`). If `podman` complains
  about a missing policy, create:

  ```sh
  mkdir -p ~/.config/containers
  printf '{"default":[{"type":"insecureAcceptAnything"}]}\n' > ~/.config/containers/policy.json
  printf 'unqualified-search-registries = ["docker.io"]\n'   > ~/.config/containers/registries.conf
  ```

## 2. Run the full stack

**With podman** (use the staged helper — see note in §6 on why):

```sh
./podman-up.sh
```

**With docker** (compose handles the ordering natively):

```sh
docker compose up -d
```

Both start the services in order and provision Kanidm automatically. Watch the
provisioning with `podman logs casino-kanidm-provision` (or
`docker compose logs -f kanidm-provision`).

When it finishes, the demo accounts already have **ready-to-use passwords**
(set via `kanidmd recover-account`):

```sh
cat kanidm/secrets/demo-credentials.txt
```

Use those username/password pairs to log in. (Alternatively, set your own
password via a one-time reset link from `kanidm/secrets/reset-links.txt` — open
the `https://kanidm:8443/ui/reset?token=...` URL in the browser; note that
opening multiple reset links in the same browser session can 403, so prefer the
ready-made credentials above.)

### Accounts created

| Account (login)  | Group           | Role in app | Notes                                           |
| ---------------- | --------------- | ----------- | ----------------------------------------------- |
| `rehmann_admin`  | `casino_admins` | `admin`     | also in `casino_users`, so also gets `user`     |
| `rehmann_user`   | `casino_users`  | `user`      |                                                 |
| `idm_admin`      | –               | (kanidm)    | recovered password in `kanidm/secrets/idm_admin.password` |

### Open the app

- Frontend: <http://localhost:8081>
- The frontend reverse-proxies `/auth`, `/user`, `/spin`, `/loan`, `/admin`
  to the backend, so the browser stays on a single origin (cookies work, no
  CORS). Login → Kanidm → back to the app.

> The published frontend image must be built with `VITE_USE_MOCK=false` (the
> GitHub workflow already does this) for it to call the real backend.

## 3. Run the pieces separately

The partial stacks share the Docker/podman network `casino_net` created by the
infra stack. Start the infra **first**:

```sh
# podman (--build builds the image from source the first time)
./podman-up.sh docker-compose.infra.yml
podman-compose -f docker-compose.backend.yml up -d --build
podman-compose -f docker-compose.frontend.yml up -d --build

# docker
docker compose -f docker-compose.infra.yml up -d
docker compose -f docker-compose.backend.yml up -d --build
docker compose -f docker-compose.frontend.yml up -d --build
```

- `docker-compose.backend.yml` reads the OAuth2 client secret and Kanidm CA
  certificate from `kanidm/secrets/` (produced by the infra provisioning), so
  let the infra stack finish provisioning before starting the backend.
- The standalone backend also registers `http://localhost:8080/auth/callback`,
  so you can drive the OIDC flow directly against the backend on port 8080.

## 4. How identity & roles flow

1. The backend is a **confidential OAuth2 client** (`m183-backend`) with PKCE.
   `provision.sh` creates it and writes its generated secret to
   `kanidm/secrets/oauth2_client_secret`; the backend reads it via
   `OIDC_CLIENT_SECRET_FILE`.
2. Group membership is mapped to a token claim by a Kanidm **claim map**:
   `casino_admins → roles=admin`, `casino_users → roles=user`.
3. The backend decodes the ID/access token and treats any role/group string
   containing `admin` as an admin (`OIDC_ADMIN_ROLE`). Admin-only endpoints
   re-check this on every request.

## 5. The Kanidm bring-up sequence

The kanidm images are **distroless** (no shell), so `busybox-init` stages a
static musl busybox into `kanidm/secrets/` first; the shell scripts then run
inside the kanidm images via that busybox. The ordered one-shots are:

```
busybox-init  → stage static busybox + applet symlinks
kanidm-cert   → kanidmd cert-generate  (offline, before the server)
kanidm        → kanidmd server         (long-running)
kanidm-recover→ kanidmd recover-account idm_admin  (online, via /data/kanidmd.sock)
kanidm-provision → kanidm CLI: groups, persons, oauth2 client, scope/claim maps
```

## 6. Useful commands & notes

```sh
# Stop (keep data) / wipe everything:
podman-compose -f docker-compose.yml down          # or: docker compose down
podman-compose -f docker-compose.yml down -v        # also wipes postgres + kanidm volumes

# Re-run provisioning (idempotent – tolerates "already exists"):
podman-compose -f docker-compose.yml up -d --no-deps kanidm-provision

# Recover any account password manually (online, server must be running):
podman exec casino-kanidm /sbin/kanidmd recover-account <name> -c /data/server.toml
```

- **podman + one-shot init containers**: `podman-compose` maps `depends_on` to
  podman's *requires*, which expects dependencies to keep **running** — our
  init containers exit on success, so a plain `podman-compose up` fails with
  "container state improper". `podman-up.sh` works around this by starting each
  stage with `--no-deps` and waiting for the one-shots to finish. Docker Compose
  honours `condition: service_completed_successfully` natively, so
  `docker compose up -d` just works.
- **Self-signed certificate**: the browser warns the first time you hit
  `https://kanidm:8443`. Accept it. The backend trusts it via `OIDC_CA_CERT`
  (`kanidm/secrets/kanidm-ca.pem`, the exported Kanidm CA).
- **`kanidm` doesn't resolve in the browser**: you forgot the `/etc/hosts`
  entry (`127.0.0.1 kanidm`).
- **Kanidm CLI version**: `provision.sh` targets the Kanidm 1.10 CLI from the
  `kanidm/tools:latest` image (`--config-path`, `--ttl`, online `recover-account`
  via the admin socket). If you pin a different Kanidm version and a step fails,
  check `podman logs casino-kanidm-provision` — the script tolerates
  "already exists", so it is safe to adjust and re-run.
- **Images**: built locally from `../backend` and `../frontend` by default
  (`grand-casino-backend:local` / `grand-casino-frontend:local`). The GitHub
  workflows in `.github/workflows/` also build and push them to Docker Hub; set
  `BACKEND_IMAGE`/`FRONTEND_IMAGE` in `.env` to pull those instead of building.
