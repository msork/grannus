#!/usr/bin/env bash
set -euo pipefail

repo="msork/grannus"

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI (gh) is required." >&2
  exit 1
fi

gh auth status

if [[ ! -d .git ]]; then
  git init -b main
fi

git add .
if ! git diff --cached --quiet; then
  git commit -m "chore: bootstrap Grannus"
fi

if git remote get-url origin >/dev/null 2>&1; then
  expected="https://github.com/${repo}.git"
  actual="$(git remote get-url origin)"
  if [[ "$actual" != "$expected" && "$actual" != "git@github.com:${repo}.git" ]]; then
    echo "Refusing to push unexpected origin: $actual" >&2
    exit 1
  fi
  git push -u origin main
else
  gh repo create "$repo" --public --source=. --remote=origin --push
fi
