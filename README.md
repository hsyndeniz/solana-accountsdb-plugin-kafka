# Solana Geyser Plugin for Kafka

Kafka publisher for use with Solana's [plugin framework](https://docs.solana.com/developing/plugins/geyser-plugins).

## Installation

#### Build

```shell
cargo build --release
```

- Linux: `./target/release/libsolana_indexer.so`
- macOS: `./target/release/libsolana_indexer.dylib`

## Config

Config is specified via the plugin's JSON config file.

### Example Config

```json
{
  "libpath": "./target/debug/libsolana_indexer.dylib",
  "account_data_notifications_enabled": true,
  "transaction_notifications_enabled": true,
  "slot_notifications_enabled": true,
  "block_notifications_enabled": true,
  "entry_notifications_enabled": true,
  "kafka_config": {
    "shutdown_timeout_ms": 30000,
    "update_account_topic": "solana.testnet.account_updates",
    "transaction_topic": "solana.testnet.transactions",
    "slot_status_topic": "solana.testnet.slot_status",
    "block_metadata_topic": "solana.testnet.block_metadata",
    "entry_notification_topic": "solana.testnet.entry_notification",
    "kafka": {
      "bootstrap.servers": "localhost:9092",
      "message.timeout.ms": "300000",
      "request.timeout.ms": "5000",
      "request.required.acks": "1",
      "compression.codec": "none"
    }
  },
  "prometheus_config": "localhost:9090",
  "commitment_level": "finalized",
  "filters": {
    "accounts": {
      "include": ["*"],
      "exclude": [
        "Sysvar1111111111111111111111111111111111111",
        "Vote111111111111111111111111111111111111111"
      ]
    },
    "programs": {
      "include": ["*"],
      "exclude": []
    },
    "transactions": {
      "mentions": ["*"]
    }
  }
}
```

# TO-DO List

- [ ] Commitment level filtering
- [ ] Kafka encoding and compression
- [ ] Ignoring programs
- [ ] Ignoring accounts
- [ ] Postgres support
- [ ] Documentation
- [ ] Tests
- [ ] CI/CD
