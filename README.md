# Subindexer API

A Rust API server that exposes redeemable subscription data.

## Setup

1. Create a `.env` file in the root directory with the content outlined in [.env.sample](./.env.sample).

2. Populate Indexer Config & Run Database + Indexer

```bash
envsubst < rindexer.yaml.template > rindexer.yaml
docker-compose up -d
```

Alternatively `make up`. See [Makefile](./.Makefile)

3. Run the API Server & Redeeming Cron Job:

```bash
# Local
cargo run
# Docker
docker run --rm --env-file .env ghcr.io/deluXtreme/subindexer-api
```

The API server will start on `http://localhost:3000/health_check`

## API Endpoints

### GET /redeemable
Returns a list of all redeemable subscriptions.

Response format:
```json
[
  {
    "id": "0x9c4412d30af600c6de7a2c746d92d63d30e67cac94946358f43422c2e08d067d",
    "subscriber": "0xcf6dc192dc292d5f2789da2db02d6dd4f41f4214",
    "recipient": "0x6b69683c8897e3d18e74b1ba117b49f80423da5d",
    "amount": "10000000000000000",
    "periods": 8,
    "category": "trusted",
    "next_redeem_at": 1753139215
  }
]
```

# Local Development

```sh
make up
```

Check for redeemable subscriptions:
```sh
curl http://localhost:3030/redeemable | jq
```

Check health
```sh
curl http://localhost:3030/health | jq
```
