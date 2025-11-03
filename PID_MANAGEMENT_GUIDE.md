# PID Management Guide - Hodei Verified Permissions

## Overview

The PID management system ensures no port conflicts when starting development servers. It automatically manages process IDs (PIDs) for Rust server and Next.js frontend.

## Quick Reference

```bash
# Start environment (with automatic cleanup)
make dev-start

# Check status of services
make dev-status

# Stop all services gracefully
make dev-stop

# Stop services and clean all PID files
make dev-clean

# View logs
make dev-logs              # Rust server logs
make dev-logs-frontend     # Next.js logs
```

## How It Works

### 1. Automatic Cleanup (dev-start)

When you run `make dev-start`, the system:

1. **Pre-start Cleanup**: Calls `manage-pids.sh prestart`
   - Reads saved PIDs from `~/.hodei-pids/`
   - Kills existing processes by PID
   - Kills processes by name pattern (safety net)

2. **Port Verification**: Checks if ports 50051 and 3000 are free
   - Fails if ports still in use after cleanup

3. **Start Services**: Launches Rust server and Next.js
   - Saves PIDs after successful start

4. **Health Checks**: Waits for services to be ready
   - Verifies ports are listening

### 2. PID Storage

PIDs are stored in `~/.hodei-pids/` directory:
- `rust-server.pid` - PID of Rust gRPC server
- `nextjs.pid` - PID of Next.js frontend

### 3. Status Display (dev-status)

Shows the status of all services:
- **RUNNING** - Process is alive and running
- **DEAD** - PID file exists but process is not running
- **NOT RUNNING** - No PID file found

### 4. Graceful Shutdown (dev-stop)

1. Reads PIDs from files
2. Sends SIGTERM to processes (graceful shutdown)
3. Waits 2 seconds
4. Sends SIG9 if process didn't stop (force kill)
5. Removes PID files

### 5. Clean Reset (dev-clean)

1. Runs `dev-stop` (kills all processes)
2. Removes `~/.hodei-pids/` directory
3. Returns to pristine state

## Manual PID Management

### Check PID files
```bash
ls -la ~/.hodei-pids/
cat ~/.hodei-pids/rust-server.pid
cat ~/.hodei-pids/nextjs.pid
```

### Check if process is running
```bash
ps -p $(cat ~/.hodei-pids/rust-server.pid)
```

### Kill process manually
```bash
kill -TERM $(cat ~/.hodei-pids/rust-server.pid)
kill -9 $(cat ~/.hodei-pids/rust-server.pid)  # Force kill
```

## Troubleshooting

### Port already in use error
```bash
# Check what's using the port
lsof -i :50051
lsof -i :3000

# Clean up everything
make dev-clean

# Restart
make dev-start
```

### Process not responding
```bash
# Force kill all
make dev-clean

# Or manually
pkill -f "hodei-verified-permissions"
pkill -f "next dev"
rm -rf ~/.hodei-pids
```

### Stale PID files
```bash
# Clean and restart
make dev-clean
make dev-start
```

## Advanced Usage

### Custom ports
```bash
RUST_PORT=8080 NEXTJS_PORT=4000 make dev-start
```

### Use PID management script directly
```bash
./scripts/manage-pids.sh prestart    # Clean before start
./scripts/manage-pids.sh status      # Show status
./scripts/manage-pids.sh kill-all    # Kill all services
```

## Benefits

✅ **No Port Conflicts**: Automatic cleanup prevents conflicts
✅ **Graceful Shutdown**: SIGTERM before SIG9
✅ **Process Visibility**: Know which PIDs are running
✅ **Automatic Cleanup**: No manual kill commands needed
✅ **Safe Restarts**: Can run dev-start multiple times
✅ **PID Tracking**: Persistent PIDs for debugging

## Files Modified

- `scripts/manage-pids.sh` - NEW: PID management script
- `scripts/dev-start-improved.sh` - MODIFIED: Uses PID management
- `Makefile` - MODIFIED: Added dev-status and dev-clean commands
- `PID_MANAGEMENT_GUIDE.md` - NEW: This guide
