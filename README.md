# Usdf-backend

Rust backend for interaction with USDF contract.

## Routes

1. POST **getSignature**: route for minting USDF. Each request invokes generation of new nonce that is stored in the persistence file

Params: 
 - user address
 - token address
 - token amount
    
Returns:
 - USDF amount
 - nonce
 - signature

2. POST **getEstimation**: route for future mint.

Params:  
 - user address
 - token address
 - token amount
    
Returns:
 - USDF amount

3. GET **get_whitelist**: route for fetching token whitelist

Returns tokens and prices for them.

## Setup

It is required to set configuration before start. Define config file in CONFIG_PATH env variable.
Or set env variables with APP_ prefix in uppercase.
Configuration file must contain fields:

```toml
log_level                           # Log level for the application layer, default: info
is_json_logging                     # Whether to use JSON logging, default: true
listener                            # Address for the listener server
redis_uri                           # Uri for redis
signing_key                         # Signing keypair string
```

## Starting dev environment

```sh
# Start dev redis instance
./scripts/redisctl.sh --start
```

## Usage

To run cli in the repo root:

```sh
cargo run
```

or to run directly the binary:

```sh
cd ./target/debug && usdf_back
```


## Stopping dev environment

```sh
# Stop dev redis instance
./scripts/redisctl.sh --stop
```
