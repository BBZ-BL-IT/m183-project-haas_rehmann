**M183 Grand Casino Rehmann**

Dieses Projekt entsteht im Rahmen der LB2 im Modul 183. Mitwirkende sind:

- Timeon Haas
- Nico Rehmann

# Dokumentation

In diesem Abschnitt sammeln und teilen wir die wichtigsten Informationen zum
Projekt.

## Überblick

Eine kleine «Casino»-Web-App: Login über einen OIDC-Identity-Provider, Spielen
an einem Spielautomaten, Kredite aufnehmen und (als Admin) Benutzer verwalten.
Das Projekt demonstriert sichere Authentisierung (OAuth2/OIDC + PKCE),
server-autoritative Spiellogik und eine containerisierte lokale Umgebung.

## Tech-Stack

| Schicht   | Technologie                                                                                                                                                          |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend  | Vue 3 + TypeScript, Vite, Pinia, vue-router, axios                                                                                                                  |
| Backend   | Rust (Edition 2024), [axum](https://github.com/tokio-rs/axum) 0.8, [sqlx](https://github.com/launchbadge/sqlx) (PostgreSQL), `axum-oidc-client` (OAuth2/OIDC + PKCE) |
| Datenbank | PostgreSQL 17                                                                                                                                                       |
| Identity  | [Kanidm](https://kanidm.com/) (OIDC-Provider)                                                                                                                       |
| DevOps    | Podman / Docker Compose, GitHub Actions → Docker Hub                                                                                                                |
| Dev-Env   | Nix-Flake (`flake.nix`)                                                                                                                                             |

## Projektstruktur

```
.
├── backend/            # Rust / axum API
│   ├── src/
│   │   ├── main.rs         # Einstiegspunkt
│   │   ├── app.rs          # Server-Bootstrap (DB-Pool, Migrationen, Layers)
│   │   ├── auth.rs         # OIDC-Client-Konfiguration
│   │   ├── identity.rs     # Benutzer + Rollen aus dem OIDC-Token extrahieren
│   │   ├── state.rs        # gemeinsamer AppState (DB-Pool, Loan-Config)
│   │   ├── routes/         # Route-Tabelle
│   │   ├── handlers/       # User- / Slot- / Admin-Handler
│   │   ├── db.rs           # sämtliches SQL (sqlx-Runtime-Queries)
│   │   ├── models.rs       # Request-/Response-DTOs (der API-Vertrag)
│   │   ├── game.rs         # server-autoritative Slot-Logik
│   │   ├── config.rs       # env-gesteuerte Konfiguration (Kreditregeln)
│   │   ├── validate.rs     # Eingabe-Validierung (Benutzernamen)
│   │   └── error.rs        # Fehlertyp → HTTP/JSON-Mapping
│   ├── migrations/         # SQL-Migrationen (in die Binary eingebettet)
│   ├── Dockerfile
│   └── .env.example        # Referenz für die Backend-Konfiguration
├── frontend/           # Vue-3-SPA
│   ├── src/
│   │   ├── api/            # typisierter API-Client + Mock-Layer + Endpoints
│   │   ├── components/     # SlotMachine, LoanButton, UserStats, AppHeader
│   │   ├── views/          # Home, Play, Admin, NotFound
│   │   ├── stores/         # Pinia-Auth-Store
│   │   ├── router/         # vue-router (Route-Guards)
│   │   └── types/          # gemeinsame DTO-Typen (Spiegel des Backends)
│   ├── Dockerfile          # baut die SPA, ausgeliefert von nginx (proxyt die API)
│   ├── nginx.conf
│   └── .env.example
├── devops/             # lokaler Container-Stack (siehe devops/README.md)
│   ├── docker-compose.yml          # gesamter Stack
│   ├── docker-compose.infra.yml    # postgres + kanidm
│   ├── docker-compose.backend.yml  # nur Backend (Docker-Hub-Image)
│   ├── docker-compose.frontend.yml # nur Frontend (Docker-Hub-Image)
│   ├── podman-up.sh                # gestaffeltes Hochfahren für Podman
│   └── kanidm/                     # Kanidm-Konfiguration + Provisioning-Skripte
├── assets/             # Diagramme
└── flake.nix           # Nix-Dev-Shell (rust, node, podman/docker, …)
```

## Architektur & Auth-Flow

Der Browser kommuniziert immer nur mit einem einzigen Origin (dem Frontend auf
`:8081`), das die API- und Auth-Pfade per Reverse-Proxy ans Backend weiterleitet.
Dadurch funktioniert das OIDC-Session-Cookie ohne CORS. Das Backend ist ein
vertraulicher OAuth2-Client (Confidential Client mit PKCE).

```
Browser ──► Frontend (nginx, :8081) ──Proxy /auth,/user,/spin,/loan,/admin──► Backend (axum, :8080)
   │                                                                              │
   └──────────── Login-Redirect ───► Kanidm (OIDC, :8443) ◄──── Token-Exchange ───┘
                                          │
                              PostgreSQL  ◄┘ (Backend persistiert den Spielstand)
```

- **Login**: `/auth` → Backend baut einen PKCE-Redirect zu Kanidm → Benutzer
  meldet sich an → Kanidm leitet zurück auf `/auth/callback` → Backend setzt das
  Session-Cookie.
- **Rollen**: Das Backend liest die Gruppen/Claims des Benutzers aus dem
  OIDC-Token; die Gruppe `casino_admins` vergibt die Rolle `admin` (bei jedem
  Admin-Aufruf erneut geprüft).
- **Spielstand** (Guthaben, Statistik, Kredite, Spins) liegt in PostgreSQL,
  aufgeteilt in `users` / `bank_accounts` / `stats` / `loans` / `spins`. Das
  Slot-Ergebnis und alle Auszahlungen entscheidet das Backend (`game.rs`),
  niemals der Client.

## Schnittstellen: Frontend ↔ Backend

Dies ist der zentrale Vertrag zwischen SPA und API. Alle Datenpfade laufen über
denselben Origin (nginx-Reverse-Proxy), sodass das **HttpOnly-Session-Cookie**
automatisch mitgesendet wird (`axios` mit `withCredentials: true`). Anfrage- und
Antwort-Bodies sind JSON.

### Wo der Vertrag im Code lebt

| Seite    | Datei                                  | Zweck                                                            |
| -------- | -------------------------------------- | ---------------------------------------------------------------- |
| Backend  | `backend/src/models.rs`                | DTOs (`serde`-(De)serialisierung) – die Quelle der Wahrheit      |
| Backend  | `backend/src/routes/mod.rs`            | Routing-Tabelle (Pfad → Handler)                                 |
| Frontend | `frontend/src/types/api.ts`            | TypeScript-Spiegel der DTOs (muss mit `models.rs` synchron sein) |
| Frontend | `frontend/src/api/endpoints.ts`        | einzige Quelle der URL-Pfade                                     |
| Frontend | `frontend/src/api/{user,slot,admin}.ts`| typisierte Aufruf-Funktionen pro Endpoint                        |
| Frontend | `frontend/src/api/client.ts`           | axios-Instanz (Cookie, Basis-URL, 401-Interceptor)              |

> Wird ein DTO im Backend geändert, **muss** `frontend/src/types/api.ts`
> nachgezogen werden – die beiden sind nicht automatisch gekoppelt.

### Endpoint-Übersicht

Alle Endpoints (ausser `/health`) verlangen eine gültige Session; ohne sie
antwortet die OIDC-Middleware mit `401 Unauthorized`. Admin-Endpoints prüfen
zusätzlich die Rolle `admin` und antworten sonst mit `403 Forbidden`.

| Methode | Pfad                       | Auth   | Request-Body            | Response (200)            | Handler                          |
| ------- | -------------------------- | ------ | ----------------------- | ------------------------- | -------------------------------- |
| GET     | `/health`                  | –      | –                       | `"ok"` (text)             | inline in `routes/mod.rs`        |
| GET     | `/user/info`               | Session| –                       | `UserInfo`                | `user_handler::get_user_info`    |
| POST    | `/spin`                    | Session| `SpinRequest`           | `SpinResponse`            | `slot_handler::spin`             |
| POST    | `/loan/{amount}`           | Session| – (Betrag im Pfad)      | `LoanResponse`            | `user_handler::take_loan`        |
| GET     | `/admin/userlist`          | Admin  | –                       | `AdminUserListResponse`   | `admin_handler::list_users`      |
| POST    | `/admin/update/user`       | Admin  | `AdminUpdateUserRequest`| `AdminUpdateUserResponse` | `admin_handler::update_user`     |
| POST    | `/admin/delete/user/{id}`  | Admin  | – (id im Pfad)          | `{ "deleted": <id> }`     | `admin_handler::delete_user`     |

Zusätzlich vom OIDC-Layer bereitgestellt (siehe Abschnitt Abhängigkeiten):

| Methode | Pfad             | Zweck                                               |
| ------- | ---------------- | --------------------------------------------------- |
| GET     | `/auth`          | startet den OAuth2/OIDC-Login (PKCE-Redirect)       |
| GET     | `/auth/callback` | OAuth2-Redirect-Ziel, tauscht Code gegen Token      |
| GET     | `/auth/logout`   | beendet die Session und leitet zum Provider weiter  |

### DTOs (Datenstruktur)

`UserInfo` (GET `/user/info`) – kompletter Profil- und Kreditstatus:

```jsonc
{
  "username": "rehmann_user",
  "roles": ["user"],            // oder ["user", "admin"]
  "balance": 1500,
  "total_spent": 800,
  "total_profit": 200,
  "highest_win_streak": 4,
  "loans_taken": 1,             // Anzahl je aufgenommener Kredite
  "loans_value": 500,          // ausstehende Kreditsumme
  "loans_in_window": 1,        // im aktuellen rollenden Fenster aufgenommen
  "loans_max": 3,              // erlaubte Kredite pro Fenster
  "loans_window_seconds": 86400,
  "loans_reset_at": null        // RFC3339-Zeitpunkt oder null, falls unter Limit
}
```

`SpinRequest` / `SpinResponse` (POST `/spin`):

```jsonc
// Request
{ "stake_amount": 10 }          // muss > 0 sein, sonst 400

// Response
{
  "reels": [7, 7, 3],          // 3 Walzen, Symbole 1..7
  "amount_earned": 0,          // vom Backend berechnete Auszahlung
  "balance": 1490,
  "total_spent": 810,
  "total_profit": 190,
  "highest_win_streak": 0
}
```

`LoanResponse` (POST `/loan/{amount}`): Betrag steht im Pfad, kein Body. Gültig
ist `1..=LOAN_MAX_AMOUNT`, sonst `400`; ist das Fensterlimit erreicht, kommt
`429 Too Many Requests`.

```jsonc
{
  "balance": 2000,
  "loans_value": 1000,
  "loans_taken": 2,
  "loans_in_window": 2,
  "loans_max": 3,
  "loans_reset_at": "2026-06-22T10:00:00+00:00"
}
```

`AdminUpdateUserRequest` (POST `/admin/update/user`): Nur die gesetzten Felder
werden geändert (partielles Update). `id` ist Pflicht.

```jsonc
{ "id": 5, "username": "neuer_name", "balance": 9999, "loans_value": 0, "loans_taken": 0 }
```

### Fehlerformat

Fehler liefern einen einheitlichen JSON-Body (`backend/src/error.rs`). Interne
Fehler werden serverseitig vollständig geloggt, aber dem Client nie offengelegt.

```jsonc
{ "error": "bad_request", "message": "loan amount must be between 1 and 10000" }
```

| Status | `error`-Code        | Auslöser                                            |
| ------ | ------------------- | --------------------------------------------------- |
| 400    | `bad_request`       | ungültige Eingabe (Einsatz ≤ 0, Kreditbetrag, …)    |
| 401    | `unauthorized`      | keine/ungültige Session (Token ungültig)            |
| 403    | `forbidden`         | Admin-Rolle erforderlich                            |
| 404    | `not_found`         | Ressource nicht gefunden                            |
| 429    | `too_many_requests` | Kreditlimit pro Fenster erreicht                    |
| 500    | `internal_error`    | Server-/DB-Fehler (Meldung wird nicht geleakt)      |

Im Frontend normalisiert `toApiError()` (`frontend/src/api/client.ts`) jeden
Axios-Fehler in das `ApiError`-Format. Bei `401` ruft ein Response-Interceptor
den global registrierten Logout-Callback auf, sodass die SPA die Session-Anzeige
zurücksetzt.

### Auth-Flow im Frontend

- Login/Logout sind **echte Browser-Redirects** (kein XHR), weil der PKCE-Flow
  eine reale Navigation zu Kanidm und zurück braucht
  (`frontend/src/api/auth.ts`).
- Beim ersten Navigieren prüft der Pinia-Store die Session, indem er
  `/user/info` aufruft (`checkSession()` in `frontend/src/stores/auth.ts`). Ein
  `401` bedeutet «keine Session».
- `vue-router`-Guards (`frontend/src/router/index.ts`) schützen `/play`
  (`requiresAuth`) und `/admin` (`requiresAdmin`); fehlt die Berechtigung, wird
  auf `/` umgeleitet. Das ist reiner UX-Schutz – die echte Durchsetzung
  geschieht serverseitig bei jedem Request.
- Mit `VITE_USE_MOCK=true` ersetzt eine Mock-Schicht (`frontend/src/api/mock.ts`)
  sämtliche Aufrufe durch lokale Dummy-Daten – nützlich für UI-Arbeit ohne
  Backend/Kanidm.

## Wichtige Abhängigkeiten

### `axum-oidc-client` (OAuth2/OIDC + PKCE)

- crates.io: <https://crates.io/crates/axum-oidc-client> · Docs:
  <https://docs.rs/axum-oidc-client>
- Verwendete Version: **0.7.0** (siehe `backend/Cargo.toml`).

Diese Crate ist das Herzstück der Authentisierung. Sie liefert ein
[tower](https://docs.rs/tower)-/axum-Middleware-Layer, das den kompletten
OIDC-Authorization-Code-Flow mit PKCE übernimmt: Login-Redirect, Callback mit
Code-gegen-Token-Tausch, verschlüsseltes Session-Cookie und Logout. Das Backend
muss dadurch weder Tokens noch den OAuth2-State selbst verwalten.

**Was die Crate bereitstellt und wo wir es nutzen:**

| Baustein                       | Herkunft                                  | Verwendung im Projekt                                                                  |
| ------------------------------ | ----------------------------------------- | -------------------------------------------------------------------------------------- |
| `OAuthConfigurationBuilder`    | `auth_builder`                            | baut die OIDC-Konfiguration aus Env-Variablen → `backend/src/auth.rs`                  |
| `AuthenticationLayer`          | `auth`                                    | das eine Layer, das alle Routen schützt + `/auth`-Routen bereitstellt → `backend/src/app.rs` |
| `AuthSession`                  | `auth_session`                            | Axum-Extractor; gibt Handlern Zugriff auf `id_token` / `access_token`                  |
| `TwoTierAuthCache` / `…Config` | `cache`                                   | zweistufiger Cache für Sessions/Token-Validierung → `backend/src/app.rs`               |
| `DefaultLogoutHandler`         | `logout::handle_default_logout`           | Standardverhalten beim Logout → `backend/src/app.rs`                                    |
| `decode_jwt_unverified`, `OidcClaims` | `jwt`                              | Claims aus dem Token lesen (Rollen/Name/E-Mail) → `backend/src/identity.rs`            |

**1. Konfiguration** (`backend/src/auth.rs`) – per Builder, gespeist aus
`backend/.env.example`. Secrets können auch aus einer Datei gelesen werden
(`*_FILE`, für Docker-Secrets). Scopes sind `openid profile email groups`; ein
optionales CA-Zertifikat (`OIDC_CA_CERT`) erlaubt das selbstsignierte Kanidm im
Dev-Setup:

```rust
let mut builder = OAuthConfigurationBuilder::default()
    .with_client_id(&client_id)
    .with_client_secret(&client_secret)
    .with_private_cookie_key(&cookie_key)   // signiert/verschlüsselt das Session-Cookie
    .with_redirect_uri(&redirect_uri)        // = OIDC_REDIRECT_URI (.../auth/callback)
    .with_authorization_endpoint(&auth_endpoint)
    .with_token_endpoint(&token_endpoint)
    .with_post_logout_redirect_uri(&post_logout_uri)
    .with_scopes(vec!["openid", "profile", "email", "groups"])
    .with_session_max_age(session_max_age);  // ACHTUNG: in MINUTEN interpretiert
```

**2. Einbindung als Layer** (`backend/src/app.rs`) – ein einziges Layer schützt
alle Routen *und* stellt die `/auth`-Endpoints (`/auth`, `/auth/callback`,
`/auth/logout`) bereit. Reihenfolge: Router → OIDC-Auth-Layer → (optional) CORS
→ Request-Tracing:

```rust
let auth_cache = Arc::new(TwoTierAuthCache::new(None, TwoTierCacheConfig::default())?);
let logout_handler = Arc::new(DefaultLogoutHandler);

let app = routes::create_router(app_state).layer(AuthenticationLayer::new(
    Arc::new(oidc_config),
    auth_cache,
    logout_handler,
));
```

**3. Nutzung im Handler** (z. B. `backend/src/handlers/user_handler.rs`) – die
`AuthSession` wird wie jeder Axum-Extractor als Funktionsparameter angefordert.
Ist keine gültige Session vorhanden, kommt der Handler gar nicht erst zur
Ausführung (das Layer antwortet mit `401`):

```rust
pub async fn get_user_info(
    State(state): State<AppState>,
    session: AuthSession,          // vom OIDC-Layer injiziert
) -> Result<Json<UserInfo>, AppError> {
    let identity = Identity::from_session(&session)?;  // siehe identity.rs
    // ...
}
```

**4. Identität & Rollen** (`backend/src/identity.rs`) – `Identity::from_session`
dekodiert die Tokens und sammelt Rollen-Strings aus konfigurierbaren Claims
(`OIDC_ROLE_CLAIMS`, Default `roles,groups,scopes,scope,entitlements`). Enthält
ein Treffer den Marker `OIDC_ADMIN_ROLE` (Default `admin`), gilt der Benutzer als
Admin. `require_admin()` ist der Guard für Admin-Handler.

> **Sicherheitshinweis:** Wir verwenden `decode_jwt_unverified`, lesen die Claims
> also ohne lokale Signaturprüfung. Das ist hier vertretbar, weil das Token
> direkt vom Token-Endpoint stammt (Backchannel, vertraulicher Client) und nicht
> aus dem Browser kommt. Die Rolle wird zudem bei **jedem** Admin-Aufruf erneut
> aus dem Token gelesen, nicht im Frontend «gemerkt».

### Weitere nennenswerte Abhängigkeiten

**Backend** (`backend/Cargo.toml`):

- [`axum`](https://docs.rs/axum) 0.8 – Web-Framework (Routing, Extractors,
  `IntoResponse`). Unser `AppError` implementiert `IntoResponse` für das
  einheitliche Fehler-JSON.
- [`sqlx`](https://docs.rs/sqlx) 0.9 (PostgreSQL, `runtime-tokio`,
  `tls-rustls`) – asynchrone DB-Zugriffe; Migrationen werden via `migrate!`
  beim Start eingebettet ausgeführt (`backend/src/app.rs`).
- [`axum-extra`](https://docs.rs/axum-extra) (`cookie`, `cookie-private`) –
  signierte/private Cookies, von der OIDC-Crate für die Session genutzt.
- [`tower-http`](https://docs.rs/tower-http) – `CorsLayer` (optional, per
  `CORS_ALLOWED_ORIGINS`) und `TraceLayer` (Request-Logging).
- [`rand`](https://docs.rs/rand) 0.9 – Zufall für die Walzen (`game.rs`).
- [`chrono`](https://docs.rs/chrono) – Zeitstempel für das rollende Kreditfenster.
- [`serde`](https://serde.rs/) / `serde_json` – (De)serialisierung der DTOs.
- [`tracing`](https://docs.rs/tracing) – strukturiertes Logging.

**Frontend** (`frontend/package.json`):

- [`vue`](https://vuejs.org/) 3 + [`vue-router`](https://router.vuejs.org/) –
  SPA-Framework und Routing (mit Auth-Guards).
- [`pinia`](https://pinia.vuejs.org/) – State-Management (Auth-Store).
- [`axios`](https://axios-http.com/) – HTTP-Client; zentral konfiguriert mit
  `withCredentials` und 401-Interceptor (`frontend/src/api/client.ts`).
- [`vite`](https://vite.dev/) + `vue-tsc` – Build/Dev-Server und Typprüfung.

## Sicherheit

### Session-Cookie

`axum-oidc-client` setzt das Session-Cookie mit `HttpOnly`, `Secure` und
`SameSite=Strict` (in der Crate fest verdrahtet, siehe `handle_callback`):

- **HttpOnly** – kein JavaScript-Zugriff → XSS kann das Cookie nicht stehlen
  (und folglich kann es auch nicht per JS gelöscht werden – siehe Logout).
- **Secure** – wird nur über HTTPS gesendet. `http://localhost` gilt im Browser
  als «secure context», daher funktioniert es in der lokalen Dev-Umgebung; in
  Produktion ist HTTPS damit Pflicht.
- **SameSite=Strict** – das Cookie wird bei Cross-Site-Requests nie mitgesendet.
  Das bietet starken **CSRF-Schutz** für die zustandsändernden POST-Endpoints
  (`/spin`, `/loan`, `/admin/*`), ein separates CSRF-Token ist deshalb nicht
  nötig.

### Logout (RP-initiated / Single Logout)

**Problem:** Ein reines lokales Logout löscht nur das Backend-Cookie und die
Server-Session, beendet aber **nicht** die SSO-Session bei Kanidm. Beim nächsten
`/auth` sieht Kanidm die noch offene Session und meldet ohne Passwortabfrage
automatisch den vorherigen Benutzer wieder an. Das Cookie ist `HttpOnly` und
kann daher auch nicht clientseitig per JavaScript entfernt werden.

**Lösung:** Das Backend nutzt den `OidcLogoutHandler` der Crate (statt
`DefaultLogoutHandler`), sobald `OIDC_END_SESSION_ENDPOINT` gesetzt ist
(`backend/src/app.rs`). Dieser Handler

1. invalidiert die Server-Session im Cache,
2. löscht das Session-Cookie (`Set-Cookie` mit Ablauf),
3. leitet den Browser an Kanidms End-Session-Endpoint weiter (mit
   `id_token_hint` + `post_logout_redirect_uri`), sodass **auch Kanidm** die
   SSO-Session beendet und der Benutzer am Ende wieder auf der App landet.

Ohne gesetzten Endpoint fällt das Backend auf das lokale Logout zurück (mit
Warnung im Log). Wichtig: Die `post_logout_redirect_uri` (App-Origin) muss beim
Kanidm-OAuth2-Client als erlaubte Logout-Redirect-URL registriert sein.

### HTTP-Security-Header & CSP

Der ausliefernde nginx (`frontend/nginx.conf`) setzt für alle Antworten:

| Header | Zweck |
| ------ | ----- |
| `Content-Security-Policy` | begrenzt Quellen für Skripte/Styles/Verbindungen → mitigiert XSS und Daten-Exfiltration |
| `X-Frame-Options: DENY` + CSP `frame-ancestors 'none'` | Clickjacking-Schutz |
| `X-Content-Type-Options: nosniff` | verhindert MIME-Type-Sniffing |
| `Referrer-Policy: no-referrer` | keine Referrer-Leaks (z. B. an Kanidm) |

Die CSP ist bewusst streng (`default-src 'self'`): Die SPA lädt Skripte/Styles
nur vom eigenen Origin und spricht per axios ausschliesslich same-origin.
`style-src 'unsafe-inline'` ist nötig, weil Vue Scoped-Styles teils als
Inline-`style`-Attribute rendert. `Strict-Transport-Security` (HSTS) ist
vorbereitet, aber auskommentiert – erst aktivieren, wenn vor nginx TLS
terminiert wird.

### Weitere Massnahmen im Code

- **SQL-Injection:** alle Queries in `db.rs` sind parametrisiert (`.bind(...)`),
  keine String-Interpolation.
- **Server-autoritative Spiellogik:** Walzen/Auszahlung/Guthaben entscheidet nur
  das Backend; `record_spin` nutzt `SELECT … FOR UPDATE` → kein Double-Spend.
- **Eingabevalidierung:** Username-Allowlist (`validate.rs`), Kreditgrenzen und
  Fensterlimit (429), Negativ-Checks bei Admin-Updates.
- **Fehler-Leakage:** interne Fehler werden geloggt, dem Client aber nur als
  «Internal server error» zurückgegeben (`error.rs`).
- **Rollenprüfung serverseitig:** die Admin-Rolle wird bei jedem Admin-Aufruf
  neu aus dem Token gelesen; die Router-Guards im Frontend sind reiner
  UX-Schutz.

### Offene Härtungspunkte (für Produktion)

- **HTTPS + HSTS** vor nginx terminieren (das `Secure`-Cookie verlangt es).
- **Admin-Rollen-Match** (`identity.rs`) prüft per Substring `admin`; jede Gruppe,
  die diese Zeichenfolge enthält, würde Admin-Rechte vergeben. Auf einen exakten
  Gruppennamen (`casino_admins`) umstellen – wurde hier noch nicht geändert, weil
  es ohne Live-Kanidm-Token nicht verifizierbar ist und ein falscher Wert den
  Admin-Login aussperren würde.
- **Port-Exposition:** Im Compose-Stack sind `5432` (Postgres) und `8080`
  (Backend) auf den Host veröffentlicht. In Produktion nicht publishen bzw. an
  `127.0.0.1` binden – die Container erreichen sich ohnehin über `casino_net`.

## Voraussetzungen

Alles Nötige liefert die Nix-Dev-Shell:

```sh
nix develop      # Rust-Toolchain, Node, Podman + Docker, OpenSSL, …
```

(Oder manuell: Rust 1.85+, Node 22+ und Podman **oder** Docker mit
Compose-Plugin.)

## Schnellstart – kompletten Stack lokal starten

Der Container-Stack (PostgreSQL + Kanidm + Backend + Frontend) liegt in
`devops/`. Details siehe [`devops/README.md`](./devops/README.md).

```sh
cd devops
cp .env.example .env          # setze einen starken PRIVATE_COOKIE_KEY

# Podman (rootless – das gestaffelte Hochfahren übernimmt die Init-Container):
./podman-up.sh
# …oder Docker:
docker compose up -d
```

Danach <http://localhost:8081> öffnen. Kanidm läuft unter
<https://localhost:8443> (einmalig das selbstsignierte Dev-Zertifikat
akzeptieren). Fertige Demo-Logins stehen in
`devops/kanidm/secrets/demo-credentials.txt` (`rehmann_admin` = Admin,
`rehmann_user` = User).

> Neue Konten werden in Kanidm administrativ angelegt (z. B.
> `kanidm person create <name> ...`) – Kanidm hat keine öffentliche
> Selbstregistrierung.

## Lokale Entwicklung (ohne Images neu zu bauen)

Infrastruktur in Containern, die Apps nativ für schnelle Iteration:

```sh
cd devops && ./podman-up.sh docker-compose.infra.yml   # nur postgres + kanidm
```

**Backend** (liest `backend/.env`; aus `backend/.env.example` kopieren):

```sh
cd backend
cargo run            # wendet beim Start Migrationen an, serviert auf :8080
cargo test           # Unit-Tests (Slot-Logik, Validierung)
```

**Frontend** (npm verwaltet die JS-Abhängigkeiten):

```sh
cd frontend
npm ci
npm run dev          # Vite-Dev-Server
npm run type-check   # vue-tsc
```

> Für UI-Arbeit ganz ohne Backend `VITE_USE_MOCK=true` in `frontend/.env`
> setzen – dann greift die eingebaute Mock-Datenschicht.

## Konfiguration

- **Backend** – `backend/.env.example` dokumentiert jede Variable: Datenbank-URL,
  OIDC-Endpoints/-Secrets (inkl. `OIDC_END_SESSION_ENDPOINT` für das
  RP-initiated Logout), Session-/Cookie-Einstellungen und die **Kreditregeln**
  (`LOAN_MAX_PER_WINDOW`, `LOAN_WINDOW_SECONDS`, `LOAN_MAX_AMOUNT`).
- **Frontend** – `frontend/.env.example`: API-Basis-URL, OIDC-Login-/Logout-Pfade
  und das `VITE_USE_MOCK`-Flag.
- **Stack** – `devops/.env.example`: Postgres-Zugangsdaten, Kanidm-Origin und der
  mit dem Backend geteilte Cookie-Key. Die Backend-/Frontend-Images werden von
  Docker Hub (`t1me0n/grand-casino-*`) bezogen, gebaut und gepusht von den
  CI-Workflows.

## Continuous Integration

`.github/workflows/` baut und pusht die Backend- und Frontend-Docker-Images bei
Pushes auf `main` zu Docker Hub, sofern das jeweilige Verzeichnis betroffen ist.

## Schnittstellen-Diagramm

![excalidraw](./assets/excalidraw.png)
