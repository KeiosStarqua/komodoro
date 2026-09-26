#!/bin/sh
set -eu
# Trunk's clap only accepts true/false. Cursor and other tools set NO_COLOR=1.
if [ -n "${NO_COLOR+x}" ]; then
  export NO_COLOR=true
fi
cd "$(dirname "$0")/../ui"
case "${1:-}" in
  serve) exec trunk serve ;;
  build) exec trunk build ;;
  *)
    echo "usage: ui.sh serve|build" >&2
    exit 1
    ;;
esac
