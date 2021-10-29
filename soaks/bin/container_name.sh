#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
#set -o xtrace

__dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOAK_ROOT="${__dir}/.."

display_usage() {
	echo -e "\nUsage: \$0 SOAK_NAME COMMIT_SHA\n"
}

if [  $# -le 1 ]
then
    display_usage
    exit 1
fi

SOAK_NAME="${1:-}"
COMMIT_SHA="${2:-}"

TAG=$("${SOAK_ROOT}/bin/container_tag.sh" "${SOAK_NAME}" "${COMMIT_SHA}")
IMAGE="vector:${TAG}"

echo "${IMAGE}"
