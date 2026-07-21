# Build frontend
FROM node:20-alpine AS frontend-build
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

FROM caddy:2-alpine
COPY --from=frontend-build /app/dist /usr/share/caddy/html
COPY docker/frontend/Caddyfile /etc/caddy/Caddyfile
COPY docker/frontend/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
