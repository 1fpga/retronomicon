FROM node:20 as frontend-builder
WORKDIR /app
COPY ./frontend /app/frontend
COPY ./package-lock.json /app/package-lock.json
COPY ./package.json /app/package.json
ENV REACT_APP_BACKEND_URL "/api/v1"
RUN npm install
RUN cd frontend && npm run build

FROM rust:latest as builder
WORKDIR /app
COPY ./retronomicon-cli /app/retronomicon-cli
COPY ./retronomicon-db /app/retronomicon-db
COPY ./retronomicon-dto /app/retronomicon-dto
COPY ./backend /app/backend
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock
COPY ./datary /app/datary
RUN cargo install --path backend

FROM debian:latest as runner
RUN apt-get update && apt-get install -y libpq-dev libssl-dev ca-certificates
COPY --from=builder /usr/local/cargo/bin/backend /usr/local/bin/backend
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
COPY ./backend/Rocket.toml /app/Rocket.toml
COPY ./certs /app/certs

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV STATIC_ROOT=/app/frontend/build
ENV DATABASE_CERTS=/app/certs/digitalocean.crt
EXPOSE 8000

WORKDIR /app
CMD ["backend"]
