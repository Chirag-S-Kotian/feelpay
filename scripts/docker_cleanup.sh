#!/usr/bin/env bash
set -euo pipefail

SCOPE="project"
FORCE="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      SCOPE="all"
      shift
      ;;
    --yes|-y)
      FORCE="true"
      shift
      ;;
    *)
      echo "Unknown arg: $1"
      echo "Usage: $0 [--all] [--yes]"
      exit 1
      ;;
  esac
done

if ! command -v docker >/dev/null 2>&1; then
  echo "docker not found"
  exit 1
fi

echo "Docker cleanup scope: $SCOPE"
if [[ "$SCOPE" == "project" ]]; then
  echo "Will remove stopped containers/images/volumes for names matching feelpay/payment only."
else
  echo "Will run global prune: containers, images, networks, build-cache, volumes."
fi

if [[ "$FORCE" != "true" ]]; then
  read -rp "Continue? (y/N): " reply
  if [[ "${reply,,}" != "y" ]]; then
    echo "Cancelled."
    exit 0
  fi
fi

if [[ "$SCOPE" == "project" ]]; then
  project_containers="$(docker ps -a --format '{{.ID}} {{.Names}}' | awk '/feelpay|payment/{print $1}')"
  if [[ -n "$project_containers" ]]; then
    echo "$project_containers" | xargs -r docker rm -f
  fi

  project_volumes="$(docker volume ls --format '{{.Name}}' | awk '/feelpay|payment/')"
  if [[ -n "$project_volumes" ]]; then
    echo "$project_volumes" | xargs -r docker volume rm
  fi

  project_images="$(docker images --format '{{.Repository}}:{{.Tag}} {{.ID}}' | awk '/feelpay|payment/{print $2}')"
  if [[ -n "$project_images" ]]; then
    echo "$project_images" | xargs -r docker rmi -f
  fi
else
  docker system prune -a -f --volumes
fi

echo "Cleanup complete."
