#!/bin/sh
# Compatibility shim for cargo-quad-apk versions that invoke the removed dx tool.
# Android build-tools ship D8 instead. javac is run with JDK 8 by cargo-quad-apk;
# D8 is launched with JDK 17 because modern D8 requires Java 11 or newer.
set -eu

out=classes.dex
minapi=1
inputs=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dex)
      shift
      ;;
    --output=*)
      out="${1#--output=}"
      shift
      ;;
    --output)
      out="$2"
      shift 2
      ;;
    --min-sdk-version)
      minapi="$2"
      shift 2
      ;;
    --min-api)
      minapi="$2"
      shift 2
      ;;
    *)
      inputs="$inputs $1"
      shift
      ;;
  esac
done

expanded=""
for inp in $inputs; do
  if [ -d "$inp" ]; then
    expanded="$expanded $(find "$inp" -name '*.class')"
  else
    expanded="$expanded $inp"
  fi
done

exec env JAVA_HOME=/opt/jdk17 PATH="/opt/jdk17/bin:$PATH" \
  "$(dirname "$0")/d8" \
  --output "$(dirname "$out")" \
  --min-api "$minapi" $expanded
