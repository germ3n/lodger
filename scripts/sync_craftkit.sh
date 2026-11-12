#!/bin/sh
# Carries the SDK and the host over to github.com/germ3n/craftkit: re-exports them into a clone of
# it (~/craftkit unless named), commits with the subject of lodger's latest commit that touched
# them, and pushes. Nothing changed, nothing is committed. See docs/sdk.md, The craftkit
# repository.
#
#     scripts/sync_craftkit.sh [destination]
set -e
cd "$(dirname "$0")/.."
DEST=${1:-$HOME/craftkit}
# Only what lodger has committed goes over, under that commit's subject.
if ! git diff --quiet HEAD -- sdk host docs/sdk.md || [ -n "$(git ls-files --others --exclude-standard -- sdk host)" ]; then
	echo "craftkit: sdk/, host/ or docs/sdk.md has changes not committed; commit them first" >&2
	exit 1
fi
if [ ! -d "$DEST/.git" ]; then
	git clone -q https://github.com/germ3n/craftkit "$DEST"
fi
MESSAGE=$(git log -1 --format=%s -- sdk host docs/sdk.md)
python3 scripts/export_craftkit.py "$DEST" > /dev/null
cd "$DEST"
git add -A
if git diff --cached --quiet; then
	echo "craftkit: nothing changed"
	exit 0
fi
git commit -q -m "$MESSAGE"
git push -q
echo "craftkit: pushed \"$MESSAGE\""
