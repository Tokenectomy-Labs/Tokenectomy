#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🚀 Publishing tokenectomy-razor to npm..."
npm publish --access public

echo "✅ tokenectomy-razor successfully published to npm!"
