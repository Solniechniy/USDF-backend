#!/usr/bin/env bash

set -eo pipefail

CONTAINER_NAME=usdf-redis
CONTAINER_IMAGE=redis:7-alpine
REDIS_HOST=${REDIS_HOST:-localhost}
REDIS_PORT=${REDIS_PORT:-16379}
REDIS_URL=redis://${REDIS_HOST}:${REDIS_PORT}

if ! [ -x "$(command -v redis-cli)" ]; then
  echo >&2 "Error: redis-cli is not installed."
  exit 1
fi

function usage() {
    cat <<EOM
Usage: $0 [COMMAND] [OPTIONS]

Commands:
    --start              Starts redis container (fails if already started)
    --restart            Restarts redis container
    --stop               Stops redis container
    --cli                Connect client
    --help               Shows this message

Options:
    --image              Container image (default: $CONTAINER_IMAGE)
    --redis-host HOST    Container host (default: localhost)
    --redis-port PORT    Container port (default: 16379)
EOM
    exit 0
}

function start() {
    RUNNING_CONTAINER=$(docker ps -a --filter name=${CONTAINER_NAME} --format '{{.ID}}')
    if [[ -n $RUNNING_CONTAINER ]]; then
        if [[ -n $REDIS_RESTART ]]; then
            stop
        else
            >&2 echo "Container '${CONTAINER_NAME}' is already running: ${RUNNING_CONTAINER}"
            exit 1
        fi
    fi

    >&2 echo -n "Starting container '${CONTAINER_NAME}': "
    docker run \
        -h $REDIS_HOST \
        -p $REDIS_PORT:6379 \
        -d \
        --name $CONTAINER_NAME \
        $CONTAINER_IMAGE

    until redis-cli -u $REDIS_URL ping 1>/dev/null 2>/dev/null; do
        >&2 echo "Redis is still unavailable - sleeping"
        sleep 1
    done

    >&2 echo "Container '${CONTAINER_NAME}' is ready at ${REDIS_URL}"
}

function stop() {
    RUNNING_CONTAINER=$(docker ps -a --filter name=${CONTAINER_NAME} --format '{{.ID}}')
    if [[ -n $RUNNING_CONTAINER ]]; then
        >&2 echo -n "Stopping container '${CONTAINER_NAME}': "
        >&2 docker rm -f $RUNNING_CONTAINER
    else
        >&2 echo "Container '${CONTAINER_NAME}' is not running"
        exit 1
    fi
}

# No script parameters
test $# -eq 0 && usage

while test $# -gt 0
do
    case "$1" in
        # Commands
        --start)
            REDIS_START=1
            ;;
        --restart)
            REDIS_RESTART=1
            ;;
        --stop)
            REDIS_STOP=1
            ;;
        --cli)
            REDIS_CLI=1
            ;;
        --help)
            usage
            ;;

        # Options
        --redis-host)
            REDIS_HOST=$2
            shift
            ;;
        --redis-port)
            REDIS_PORT=$2
            shift
            ;;
        --*) echo "Invalid option $1"
            exit 1
            ;;
    esac
    shift
done

if [[ -n "${REDIS_START}" ]]; then
    start
elif [[ -n "${REDIS_RESTART}" ]]; then
    start
elif [[ -n "${REDIS_STOP}" ]]; then
    stop
elif [[ -n "${REDIS_CLI}" ]]; then
    redis-cli -u $REDIS_URL
fi
