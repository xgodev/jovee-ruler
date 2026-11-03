#!/usr/bin/env bash
# Responsibility: moves the checkout's `branch =` git libraries to their branch tip.
#
# Run before a build from a clone, so the app never builds an older
# JoVee Theme than its main.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
# shellcheck source=follow-branches.sh
. scripts/follow-branches.sh
follow_branches
