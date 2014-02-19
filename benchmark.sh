#!/bin/bash
# Script for benchmarking the zhtta server

LOGFILE="zhtta-perf.log"
RATE=50
CONNECTIONS=100
SENDBUFFER=4096
RECVBUFFER=16384
SERVER="127.0.0.1"
PORT=4414
TEST_FILE="./zhtta-test.txt"
USE_TEST_FILE=0
LOGFILE_SEP="------------------------------\n"

function benchmark () {
    # Add the current commit id, git user name, and date/time to the logfile.
    printf "%b" $LOGFILE_SEP >> $LOGFILE
    echo $(git config --get user.name) $(git rev-parse HEAD) | tee -a $LOGFILE
    echo $(date) | tee -a $LOGFILE
    printf "%b" $LOGFILE_SEP >> $LOGFILE

    # Benchmark the server
    (case $USE_TEST_FILE in
        "1" )
            echo "Using test file for benchmark."
            httperf --client=0/1 --server=$SERVER --port=$PORT --rate=$RATE \
                --send-buffer=$SENDBUFFER --recv-buffer=$RECVBUFFER \
                --num-conns=$CONNECTIONS \ --num-calls=1 --wlog=y,"$TEST_FILE"
            ;;
        * )
            echo "Using web root for benchmark."
            httperf --client=0/1 --server=$SERVER --port=$PORT --uri=/ \
                --rate=$RATE --send-buffer=$SENDBUFFER --recv-buffer=$RECVBUFFER \
                --num-conns=$CONNECTIONS \ --num-calls=1
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
}


while [[ $# > 1 ]]
do
    opt="$1";
    shift; #expose next argument
    case "$opt" in
        "--" ) 
            break 2
            ;;
        "--port")
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
        "--testfile")
            TEST_FILE="$1"
            USE_TEST_FILE=1
            ;;
        "--help" )
            help
            ;;
        * )
            ;;
    esac
done

if [ -z "$HELP" ]; then
    start_benchmark
fi
