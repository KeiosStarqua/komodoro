#!/bin/sh
set -eu
cd "$(dirname "$0")/../ui"
case "${1:-}" in
  serve) exec trunk serve ;;
  build) exec trunk build ;;
  *)
    echo "usage: ui.sh serve|build" >&2
    exit 1
    ;;
esac
