# Add endpoint to expose daemon binary location

## Summary

Add a new gRPC endpoint (or extend GetDaemonInfo) that returns the path to the running daemon binary, and display it in the CLI TUI.

## Motivation

- Debugging which binary version is actually running
- CLI tooling that needs to know the daemon location
- Upgrade/update workflows that need to replace the binary
- Diagnostics and troubleshooting

## Proposed Solution

1. **Daemon**: Use `std::env::current_exe()` in Rust to retrieve the absolute path to the running daemon executable
2. **CLI TUI**: Display the daemon binary path in the daemon info/status screen

## Acceptance Criteria

- [ ] Daemon exposes its binary path via gRPC endpoint
- [ ] CLI TUI shows the daemon binary path (e.g., in `centy daemon info` or status view)
