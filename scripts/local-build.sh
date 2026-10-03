#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
HOST=127.0.0.1
PORT=3000
BUILD_ONLY=false
INSTALL=true
CARGO_ARGS=(--locked -p liturgy-backend)

usage() {
  printf '%s\n' \
    'Usage: ./scripts/local-build.sh [options]' \
    '' \
    'Build the frontend and backend, then serve both locally.' \
    '' \
    '  --release       Build an optimized Rust binary (default: debug)' \
    '  --build-only    Build without starting the server' \
    '  --skip-install  Reuse installed frontend dependencies instead of npm ci' \
    '  --host HOST     Bind address (default: 127.0.0.1)' \
    '  --port PORT     Listen port (default: 3000)' \
    '  --help          Show this help' \
    '' \
    'Requires Cargo and Node.js ^20.19.0 or >=22.12.0. Nothing is published.'
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

while (( $# > 0 )); do
  case "$1" in
    --release) CARGO_ARGS+=(--release); shift ;;
    --build-only) BUILD_ONLY=true; shift ;;
    --skip-install) INSTALL=false; shift ;;
    --host|--port)
      (( $# >= 2 )) && [[ -n "$2" && "$2" != --* ]] || fail "$1 requires a value"
      if [[ "$1" == --host ]]; then HOST=$2; else PORT=$2; fi
      shift 2
      ;;
    --help|-h) usage; exit 0 ;;
    *) fail "Unknown option: $1 (see --help)" ;;
  esac
done

[[ "$PORT" =~ ^[0-9]{1,5}$ ]] || fail 'Port must be an integer between 1 and 65535'
PORT=$((10#$PORT))
(( PORT >= 1 && PORT <= 65535 )) || fail 'Port must be between 1 and 65535'

for executable in cargo npm node; do
  command -v "$executable" >/dev/null 2>&1 || fail "Required command not found: $executable"
done

node --input-type=module -e '
  const [major, minor] = process.versions.node.split(".").map(Number);
  if (!((major === 20 && minor >= 19) || (major === 22 && minor >= 12) || major > 22)) {
    console.error(`Unsupported Node.js ${process.versions.node}; use ^20.19.0 or >=22.12.0`);
    process.exit(1);
  }
'

if [[ "$BUILD_ONLY" == false ]]; then
  node --input-type=module -e '
    import net from "node:net";
    const [host, port] = process.argv.slice(1);
    const server = net.createServer();
    server.once("error", error => {
      console.error(`Cannot listen on ${host}:${port}: ${error.message}. Choose another --port or --host.`);
      process.exitCode = 1;
    });
    server.listen(Number(port), host, () => server.close());
  ' "$HOST" "$PORT"
fi

cd "$ROOT"
if [[ "$INSTALL" == true ]]; then
  printf '\nInstalling frontend dependencies from the lockfile...\n'
  npm --prefix "$ROOT/liturgy-frontend" ci
fi

printf '\nBuilding frontend...\n'
npm --prefix "$ROOT/liturgy-frontend" run build

printf '\nBuilding backend...\n'
cargo build "${CARGO_ARGS[@]}"

if [[ "$BUILD_ONLY" == true ]]; then
  printf '\nLocal build complete. No server started.\n'
  exit 0
fi

URL_HOST=$HOST
if [[ "$HOST" == 0.0.0.0 || "$HOST" == :: ]]; then
  URL_HOST=127.0.0.1
elif [[ "$HOST" == *:* ]]; then
  URL_HOST="[$HOST]"
fi
printf '\nStarting local build at http://%s:%s/\nPress Ctrl+C to stop.\n' "$URL_HOST" "$PORT"
exec cargo run "${CARGO_ARGS[@]}" -- \
  --host "$HOST" \
  --port "$PORT" \
  --calendar-data-dir "$ROOT/calendar_calc/calendar_data" \
  --ordo-rules-dir "$ROOT/ordo/rules" \
  --frontend-dir "$ROOT/liturgy-frontend"