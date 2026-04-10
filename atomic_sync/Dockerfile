# --- Build stage ---
FROM node:24-slim AS build

WORKDIR /app
COPY package.json package-lock.json* ./
COPY packages/shared/package.json packages/shared/
COPY packages/api/package.json packages/api/
COPY packages/web/package.json packages/web/

RUN npm install

COPY tsconfig.base.json ./
COPY packages/shared/ packages/shared/
COPY packages/api/ packages/api/
COPY packages/web/ packages/web/

RUN npm run build -w packages/web

# --- API ---
FROM node:24-slim AS api

WORKDIR /app
COPY --from=build /app/package.json /app/package-lock.json* ./
COPY --from=build /app/packages/shared/package.json packages/shared/
COPY --from=build /app/packages/api/package.json packages/api/
COPY --from=build /app/packages/shared/ packages/shared/
COPY --from=build /app/packages/api/ packages/api/

RUN npm install --omit=dev

EXPOSE 3000
CMD ["node", "--import", "tsx", "packages/api/src/index.ts"]

# --- Web (static files served by nginx) ---
FROM nginx:alpine AS web

COPY --from=build /app/packages/web/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
