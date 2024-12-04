# Stage 1: Сборка бэкенда
FROM rust:latest AS backend-builder
WORKDIR /app/backend
COPY backend/ .
RUN cargo build --release

# Stage 2: Сборка фронтенда
FROM node:22 AS frontend-builder
WORKDIR /app/frontend
COPY frontend/ .
RUN npm install
RUN npm run build

# Stage 3: Финальный образ
FROM debian:bullseye-slim
WORKDIR /app

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