#!/usr/bin/env bash

# List of packages to check
packages=(
  "backslash"
  "chalk-template"
  "supports-hyperlinks"
  "has-ansi"
  "simple-swizzle"
  "color-string"
  "error-ex"
  "color-name"
  "is-arrayish"
  "slice-ansi"
  "color-convert"
  "wrap-ansi"
  "ansi-regex"
  "supports-color"
  "strip-ansi"
  "chalk"
  "debug"
  "ansi-styles"
)

# Check argument
if [ -z "$1" ]; then
  echo "Usage: $0 /path/to/package-lock.json"
  exit 1
fi

lockfile="$1"

# Loop through the packages and grep for them
for pkg in "${packages[@]}"; do
  grep -n "\"$pkg\"" "$lockfile"
done