#!/bin/sh
# Run the Android D8 tool with Java 17 while cargo-quad-apk/javac use Java 8.
set -eu
D8_REAL="$(dirname "$0")/d8.original"
if [ ! -x "$D8_REAL" ]; then
  echo "D8 original executable missing: $D8_REAL" >&2
  exit 127
fi
exec env JAVA_HOME=/opt/jdk17 PATH="/opt/jdk17/bin:$PATH" "$D8_REAL" "$@"
