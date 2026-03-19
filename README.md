# lmssh - Language Model SSH Honeypot

> English | [简体中文](README_zh.md)

lmssh is an SSH honeypot server written in Rust that simulates a realistic shell environment while integrating OpenAI's language models to generate intelligent responses. It serves as a deceptive server for security research and monitoring, logging all session activities for analysis.

## Features

- **SSH Server**: Full SSHv2 server supporting password authentication (public key rejected)
- **Interactive Shell**: Line editing, command history, and realistic prompt simulation
- **Built-in Commands**: `cd`, `pwd`, `clear`, `history`, `exit`, `logout`
- **Command Blacklist**: Blocks interactive TUI commands (vim, top, tmux, etc.) with "command not found" responses
- **AI Integration**: Forwards unsupported commands to OpenAI-compatible APIs with streaming responses
- **Output Guard**: Protects against runaway outputs with configurable limits on length, lines, and numbered lines
- **Virtual File System**: Simulates file system operations (mkdir, touch, rm, redirections) without executing real commands
- **Session Logging**: Comprehensive JSONL logging of connection, authentication, command, and AI response events
- **Configurable**: TOML-based configuration with sensible defaults

## Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- OpenAI API key (or compatible endpoint)

### Installation

```bash
git clone <repository-url>
cd lmssh
cargo build --release
```

### Configuration

Create a configuration file at `~/.config/lmssh/config.toml`:

```toml
[ssh]
listen_addr = "0.0.0.0:2222"
hostname = "debian"
banner = "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3"
host_key_path = "data/host_ed25519"
session_timeout_seconds = 120
max_input_rate_per_second = 32

[limits]
max_output_length = 8192
max_output_lines = 200
max_numbered_lines = 15

[[users]]
username = "root"
password = "password"

[openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
max_tokens = 512
temperature = 0.6

[logging]
dir = "logs"
```

### Running

```bash
cargo run -- -c config.toml
```

Connect via SSH:

```bash
ssh root@127.0.0.1 -p 2222
# Password: password
```

## Usage

Once connected, you'll see a realistic shell prompt:

```
root@debian:/root#
```

- Built-in commands work as expected
- Blacklisted commands return "command not found"
- Other commands trigger AI-generated responses streamed in real-time
- File system operations are simulated in a virtual environment

## Architecture

The project is organized into modular components:

- **config**: Configuration loading and defaults
- **ssh**: SSH server implementation using russh
- **session**: Session state, command routing, and output guarding
- **terminal**: Line editor for interactive input
- **vfs**: Virtual file system with side-effect simulation
- **ai**: OpenAI client for streaming chat completions
- **logging**: JSONL event logging per session
- **prompt**: System prompt construction for AI interactions

## Development

### Building

```bash
cargo check --all-targets
cargo clippy
cargo test
```

### Testing

Comprehensive test suite covers:

- SSH handler smoke tests
- Configuration loading
- Command routing and blacklisting
- AI request building and streaming
- Session state management
- Output guard limitations

Run all tests:

```bash
cargo test
```

### Code Quality

The project enforces strict quality standards:

- No warnings allowed (except explicit `#[allow]` with justification)
- All tests must pass
- Clippy checks for idiomatic Rust
- Modular design with clear separation of concerns

## Logging

Session events are logged to JSONL files in the configured directory (`logs/` by default). Each session creates a separate file with events including:

- `Connected`: Client connection
- `AuthSuccess`/`AuthFailed`: Authentication attempts
- `Command`: Commands entered by users
- `AiResponse`: AI-generated responses with byte counts
- `Disconnected`: Session termination

## Security Considerations

- **No real command execution**: All commands are either built-in, blacklisted, or forwarded to AI
- **Virtual file system**: File operations are simulated without touching the host filesystem
- **Output limiting**: Prevents denial-of-service through excessive AI output
- **Session isolation**: Each connection operates in its own virtual environment
- **Controlled authentication**: Only password authentication supported with configurable credentials

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

## Contributing

[Contribution guidelines](CONTRIBUTING.md)
