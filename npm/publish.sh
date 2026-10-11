#!/usr/bin/env bash
# Publishes the directories build.mjs printed, platform packages first so the
# launcher never points at a version that does not exist yet. A version that is
# already on the registry is skipped, so a re-run after a partial failure is safe.
set -euo pipefail
for dir in "$@"; do
  name="$(node -p "require(require('path').resolve('${dir}', 'package.json')).name")"
  version="$(node -p "require(require('path').resolve('${dir}', 'package.json')).version")"
  if npm view "${name}@${version}" version >/dev/null 2>&1; then
    echo "${name}@${version} is already published"
    continue
  fi
  (cd "${dir}" && npm publish --access public ${NPM_PUBLISH_FLAGS:-})
done
