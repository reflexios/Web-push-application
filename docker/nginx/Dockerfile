# Build frontend
FROM node:20-alpine AS frontend-build
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .

RUN npm run build

FROM nginx:1.27-alpine
COPY --from=frontend-build /app/dist /usr/share/nginx/html
COPY docker/nginx/push.conf /etc/nginx/conf.d/default.conf
