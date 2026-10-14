# Statische Produktseite mit Download. Baut GitHub Actions (siehe .github/workflows/website.yml).
FROM nginx:1.29-alpine
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY website/ /usr/share/nginx/html/
HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO- http://127.0.0.1/ >/dev/null || exit 1
