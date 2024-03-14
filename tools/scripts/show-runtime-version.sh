#!/bin/sh

# Quick script to output the runtime spec_version at a particular commit.

# set -ex

if [[ -z "$1" ]]; then
  cat <<-EOF
Usage: $0 <git commit sha>
EOF

  exit 1
fi

(git cat-file -p "${1}:runtime/frequency/src/lib.rs" | grep "spec_version:") 2>/dev/null \
|| (git cat-file -p "${1}:runtime/mrc/src/lib.rs" | grep "spec_version:" )
