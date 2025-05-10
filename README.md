# bsky-rmcp

A Model Context Protocol (MCP) server implementation for Bluesky, written in Rust.

## Overview

This project provides an MCP-compatible server for the Bluesky social network, using the official [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk).

- **Stdio-based**: The server currently communicates via standard input/output (stdio) only.
- **Environment Variables**: Requires two environment variables to be set for authentication:
  - `BLUESKY_IDENTIFIER`: Your Bluesky handle or DID
  - `BLUESKY_APP_PASSWORD`: Your Bluesky app password

## Usage

1. Set the required environment variables:

   ```sh
   export BLUESKY_IDENTIFIER="your-handle-or-did"
   export BLUESKY_APP_PASSWORD="your-app-password"
   ```

2. Build and run the server:

   ```sh
   cargo run --bin bsky-rmcp
   ```

The server will start and communicate over stdio, ready to be used as an MCP server.

## License

See [LICENSE](LICENSE).
