#!/bin/bash

# Test and Coverage Script for ca-miner
# This script runs tests and generates coverage reports

set -e

echo "🧪 Running unit tests..."
cargo test

echo ""
echo "📊 Generating coverage report..."
cargo tarpaulin \
    --verbose --all-features \
    --workspace --timeout 120 \
    --out Xml --out Html --out stdout \
    --output-dir coverage

echo ""
echo "✅ Coverage analysis complete!"
echo "📁 Reports generated:"
echo "  - coverage/cobertura.xml (XML format)"
echo "  - coverage/tarpaulin-report.html (HTML format)"
echo ""
echo "🌐 Open the HTML report:"
echo "  open coverage/tarpaulin-report.html"
