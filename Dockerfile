# Stage 1: Сборка бэкенда
FROM rust:latest AS backend-builder
WORKDIR /app/backend
COPY backend/ .

ARG DATABASE_URL
ARG ACCESS_TOKEN_SECRET
ARG REFRESH_TOKEN_SECRET
ENV DATABASE_URL=$DATABASE_URL
ENV ACCESS_TOKEN_SECRET=$ACCESS_TOKEN_SECRET
ENV REFRESH_TOKEN_SECRET=$REFRESH_TOKEN_SECRET

RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres
RUN sqlx migrate
RUN cargo sqlx prepare

RUN cargo build --release

# Stage 2: Сборка фронтенда
FROM node:22 AS frontend-builder
WORKDIR /app/frontend
COPY frontend/ .
RUN npm install
RUN npm run build

# Stage 3: Финальный образ
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Копируем бэкенд
COPY --from=backend-builder /app/backend/target/release/backend ./backend

# Копируем фронтенд
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Устанавливаем переменные окружения
ENV PORT=8000

# Открываем порт
EXPOSE 8000

# Команда запуска
CMD ["./backend"]
