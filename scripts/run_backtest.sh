#!/bin/bash
# Run backtest
# Usage: ./scripts/run_backtest.sh XAUUSD

INSTRUMENT=${1:-XAUUSD}
echo "Running backtest for $INSTRUMENT..."
cargo run --release -- "$INSTRUMENT" --backtest
