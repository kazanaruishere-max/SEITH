#!/bin/bash
# Run paper trading
# Usage: ./scripts/run_paper_trade.sh XAUUSD

INSTRUMENT=${1:-XAUUSD}
echo "Running paper trading for $INSTRUMENT..."
cargo run --release -- "$INSTRUMENT" --paper
