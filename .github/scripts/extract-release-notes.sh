#!/usr/bin/env bash
set -euo pipefail

tag=${1:?usage: extract-release-notes.sh TAG [CHANGELOG]}
changelog=${2:-CHANGELOG.md}
version=${tag#v}

awk -v version="$version" '
BEGIN {
  heading = "## [" version "]"
}
index($0, heading) == 1 &&
  (length($0) == length(heading) ||
   substr($0, length(heading) + 1, 1) == " ") {
  found = 1
  in_section = 1
  next
}
in_section && /^## \[/ {
  exit
}
in_section {
  print
}
END {
  if (!found) {
    print "missing CHANGELOG section for " version > "/dev/stderr"
    exit 1
  }
}
' "$changelog"
