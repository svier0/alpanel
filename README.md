# Alpanel

A lightweight server management panel.
Designed specifically for Alpine.
Developed using Rust.

## Install

```bash
sudo wget -O install.sh https://raw.githubusercontent.com/svier0/alpanel/master/scripts/install.sh && sudo bash install.sh
```

For users in China:

```bash
sudo wget -O install.sh https://gh-proxy.com/https://raw.githubusercontent.com/svier0/alpanel/master/scripts/install.sh && sudo bash install.sh
```

## Structure

```
alpanel/
├── backend/    Rust + Axum backend
├── frontend/   Vue 3 + Vite frontend
└── scripts/    Management scripts (alp.sh) & install scripts
```

## Development

```bash
# Backend
cd backend && cargo run

# Frontend
cd frontend && bun install && bun run dev
```

## Management (alp.sh)

The `scripts/alp.sh` script provides panel service management on the server:

| Command    | Action             |
|------------|--------------------|
| `alp 11`   | Start panel        |
| `alp 12`   | Stop panel         |
| `alp 13`   | Restart panel      |
| `alp 21`   | Change username    |
| `alp 22`   | Change password    |
| `alp 31`   | Change port        |
