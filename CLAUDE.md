# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

bsky-rmcp is a Model Context Protocol (MCP) server implementation for Bluesky social network, written in Rust. It provides an MCP-compatible interface to interact with Bluesky's API using stdio communication.

## Development Commands

**Build and run:**
```bash
cargo run --bin bsky-rmcp
```

**Build only:**
```bash
cargo build
```

**Development commands:**
```bash
cargo check           # Check compilation
cargo clippy --all-targets  # Run linting
cargo test            # Run tests
cargo fmt             # Format code
```

## Environment Setup

The application requires these environment variables:
- `BLUESKY_IDENTIFIER`: Bluesky handle or DID for authentication
- `BLUESKY_APP_PASSWORD`: Bluesky app password

## Architecture

The codebase follows a modular structure:

- **`src/lib.rs`**: Main library entry point, exports BskyService
- **`src/service.rs`**: Core BskyService implementation with MCP tool handlers for Bluesky operations
- **`src/types.rs`**: Parameter structs and enums for API operations with JSON schema definitions
- **`src/utils.rs`**: Utility functions for data conversion and API helpers
- **`src/bin/main.rs`**: Binary entry point handling authentication and stdio transport

**Key Components:**
- Uses `bsky-sdk` for Bluesky API integration
- Uses `rmcp` (Model Context Protocol Rust SDK) for MCP server functionality
- Implements tools for profile management, feed retrieval, notifications, and post creation
- Supports both regular posts and reply posts with proper thread handling
- Includes a prompt system for common workflows like viewing self feed

**Tool Categories:**
- Profile operations: `get_did`, `get_profile`
- Feed operations: `get_author_feed`, `get_post_thread`, `search_posts`
- Notification operations: `list_notifications`, `get_unreplied_mentions`
- Content creation: `create_post` (supports replies and rich text)

The service runs as an MCP server over stdio, making it suitable for integration with MCP-compatible clients.