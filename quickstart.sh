#!/bin/bash

echo "🚀 Pipecheck - Quick Start Guide"
echo "================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed"
    echo "📦 Install Rust from: https://rustup.rs"
    exit 1
fi

echo "✅ Rust is installed"
echo ""

# Build the project
echo "🔨 Building pipecheck..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    
    # Run tests
    echo "🧪 Running tests..."
    cargo test
    
    if [ $? -eq 0 ]; then
        echo "✅ All tests passed!"
        echo ""
        
        # Demo
        echo "🎬 Running demo..."
        echo ""
        echo "Example 1: Valid workflow"
        echo "-------------------------"
        ./target/release/pipecheck tests/fixtures/github/valid.yml
        echo ""
        
        echo "Example 2: Circular dependency (will fail)"
        echo "------------------------------------------"
        ./target/release/pipecheck tests/fixtures/github/circular.yml || true
        echo ""
        
        echo "Example 3: JSON output"
        echo "---------------------"
        ./target/release/pipecheck tests/fixtures/github/valid.yml --format json
        echo ""
        
        echo "✅ Setup complete!"
        echo ""
        echo "Next steps:"
        echo "1. Install globally: cargo install --path ."
        echo "2. Run on your workflows: pipecheck .github/workflows/ci.yml"
        echo "3. Integrate into CI: see README.md"
        echo ""
        echo "To publish:"
        echo "1. Read PUBLISHING_CHECKLIST.md"
        echo "2. Setup GitHub repository"
        echo "3. Create release tag: git tag v0.1.0 && git push --tags"
    else
        echo "❌ Tests failed"
        exit 1
    fi
else
    echo "❌ Build failed"
    exit 1
fi
