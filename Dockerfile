FROM node:20 as frontend-builder
WORKDIR /app
COPY . .
ENV REACT_APP_BACKEND_URL "/api/v1"
RUN npm install
RUN cd frontend && npm run build

FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo install --path backend

FROM debian:latest as runner
RUN apt-get update && apt-get install -y libpq-dev libssl-dev ca-certificates
COPY --from=builder /usr/local/cargo/bin/backend /usr/local/bin/backend
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
COPY ./backend/Rocket.toml /app/Rocket.toml

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV STATIC_ROOT=/app/frontend/build
EXPOSE 8000

WORKDIR /app
CMD ["backend"]
