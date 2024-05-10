# Helix BTC Bridge

## Introduction

This project facilitates cross-chain fund transfers from **Bitcoin** to **X** chain.

It is built using three main components:
- User Interface (UI)
- Off-chain Relayer
- X Chain Contract

### Technical Architecture

```mermaid
sequenceDiagram
    participant U as User
    participant B as Bitcoin
    participant R as Off-chain Relayer
    participant X as X Chain
    U->>B: Transfer to Helix vault address with \`OP_RETURN\`,<br>contains an X ID and an address on X
    R->>B: Monitor tx
    R->>X: Submit tx info
    X->>R: Verify tx info
    X->>U: Release funds to target address on X
```

## Component Overview

### UI

TODO.

### Relayer

TODO.

### Contract

TODO.

## License

State the type of license the project is under, [GPL-3.0](./LICENSE).
