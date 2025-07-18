# Subindexer API

A Rust API server that exposes redeemable subscription data.

## Setup

1. Create a `.env` file in the root directory with the following content:
```
DATABASE_URL=postgres://username:password@localhost:5432/your_database
```

2. Install dependencies:
```bash
cargo build
```

3. Run the server:
```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### GET /redeemable
Returns a list of all redeemable subscriptions.

Response format:
```json
[
  {
    "sub_id": "string",
    "module": "string",
    "subscriber": "string",
    "recipient": "string",
    "amount": 0,
    "next_redeem_at": 0
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
