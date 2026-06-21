# --- Build-Stage: Vue/Vite-App bauen ---
FROM node:22-alpine AS build
WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

# Build-Zeit-Variablen (Vite backt sie ins Bundle). Defaults sind hier gesetzt,
# koennen aber per --build-arg ueberschrieben werden (siehe Workflow).
# Leerer API-Base => relative Calls an denselben Origin (nginx proxyt ans Backend).
ARG VITE_API_BASE_URL=
# OIDC-Pfade des Backends (axum-oidc-client base_path = /auth).
ARG VITE_OAUTH_LOGIN_PATH=/auth
ARG VITE_OAUTH_LOGOUT_PATH=/auth/logout
ARG VITE_USE_MOCK=false
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL \
    VITE_OAUTH_LOGIN_PATH=$VITE_OAUTH_LOGIN_PATH \
    VITE_OAUTH_LOGOUT_PATH=$VITE_OAUTH_LOGOUT_PATH \
    VITE_USE_MOCK=$VITE_USE_MOCK

RUN npm run build

# --- Serve-Stage: statisches Bundle via nginx ausliefern ---
FROM nginx:alpine AS production
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]