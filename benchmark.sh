#!/bin/bash
# Script for benchmarking the zhtta server

LOGFILE="zhtta-perf.log"
RATE=10000
CONNECTIONS=10000
SENDBUFFER=4096
RECVBUFFER=16384
SERVER="127.0.0.1"
PORT=4414
TEST_FILE="./zhtta-test.txt"
URI=""
USE_TEST_FILE=0
LOGFILE_SEP="------------------------------\n"

function benchmark () {
    # Add the current commit id, git user name, and date/time to the logfile.
    printf "%b" $LOGFILE_SEP >> $LOGFILE
    echo $(git config --get user.name) $(date) | tee -a $LOGFILE
    echo $(git rev-parse HEAD) $(git symbolic-ref -q HEAD) | \
        tee -a $LOGFILE
    printf "%b" $LOGFILE_SEP >> $LOGFILE

    # Benchmark the server
    (case $USE_TEST_FILE in
        "1" )
            /usr/bin/time -a -o $LOGFILE \
            httperf --client=0/1 --server=$SERVER --port=$PORT --rate=$RATE \
                --send-buffer=$SENDBUFFER --recv-buffer=$RECVBUFFER \
                --num-conns=$CONNECTIONS --num-calls=1 --wlog=y,"$TEST_FILE"
            ;;
        * )
            /usr/bin/time -a -o $LOGFILE \
            httperf --client=0/1 --server=$SERVER --port=$PORT --uri=$URI \
                --rate=$RATE --send-buffer=$SENDBUFFER --recv-buffer=$RECVBUFFER \
                --num-conns=$CONNECTIONS --num-calls=1
            ;;
    esac) | tee -a $LOGFILE;
    echo "Wrote performance benchmark to '$LOGFILE'."
}

function start_benchmark () {
    # Only start the server if it's not already started
    pidof zhtta &> /dev/null
    if [ "0" -ne "$?" ]; then
        RUST_LOG="zhtta=debug";
        echo "Running benchmark."
        (make) && (./zhtta --ip $SERVER --port $PORT &) && (benchmark);
        (kill -15 $(pidof zhtta))
    else 
        echo "The server is already started. Try killing it first."
    fi
}

function help () {
    HELP=1
    echo >&2 "Benchmark the zhtta webserver"
    echo >&2 ""
    echo >&2 "Command-line arguments:"
    echo >&2 "  --port PORT         Use port PORT"
    echo >&2 "  --server SERVER     Run the server at address SERVER"
    echo >&2 "  --connections CONN  Specify the total number CONN of connections to create"
    echo >&2 "  --rate RATE         Specify the fixed rate RATE at which connections are"
    echo >&2 "                      created"
    echo >&2 "  --testfile FILE     Specify a file FILE to use as httperf testfile input"
    echo >&2 "  --uri URI           Specify a parituclar uri URI to benchmakr"
}

while [[ $# > 0 ]]; do
    opt="$1";
    shift;
    case "$opt" in
        "--port" )
            PORT="$1"
            ;;
        "--server" )
            SERVER="$1"
            ;;
        "--connections" )
            CONNECTIONS="$1"
            ;;
        "--rate" )
            RATE="$1"
            ;;
        "--testfile" )
            if [ -z "$URI" ]; then
                TEST_FILE="$1"
                USE_TEST_FILE=1
            else
                ERR=1
                echo >&2 "--uri and --testfile are mutually exclusive"
            fi
            ;;
        "--uri" )
            if [ "$USE_TEST_FILE" != 1 ]; then
                URI="$1"
            else
                ERR=1
                echo >&2 "--uri and --testfile are mutually exclusive"
            fi
            ;;
        "--help" )
            help
            ;;
        * )
            ;;
    esac
done

if [ -z "$HELP" -a -z "$ERR" ]; then
    if [ -z "$URI" ]; then
        URI="/"
    fi
    start_benchmark
fi
