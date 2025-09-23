# SafeHold Docker Setup

Simple Docker Compose setup for SafeHold environment variable manager.

## Quick Start

```bash
# Build and start the container
docker-compose up -d

# Execute SafeHold commands
docker-compose exec safehold safehold about
docker-compose exec safehold safehold create "my-project"
docker-compose exec safehold safehold add --project "my-project" --key "DATABASE_URL" --value "postgresql://localhost:5432/mydb"
docker-compose exec safehold safehold get --project "my-project" --key "DATABASE_URL"
docker-compose exec safehold safehold list --project "my-project"
docker-compose exec safehold safehold list-projects

# Interactive shell
docker-compose exec safehold bash

# View logs
docker-compose logs safehold

# Stop and remove
docker-compose down
```

## Features

- **Persistent Data**: Data stored in named volume `safehold_data`
- **Security**: Runs as non-root user
- **Minimal Image**: Multi-stage build for small footprint
- **Health Checks**: Built-in container health monitoring

## Data Location

SafeHold data is persisted in a Docker volume at `/app/data` inside the container.

## Environment Variables

- `SAFEHOLD_DATA_DIR`: Data storage directory (default: `/app/data`)
- `RUST_LOG`: Logging level (default: `info`)