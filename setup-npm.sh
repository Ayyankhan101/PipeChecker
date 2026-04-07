#!/bin/bash

# Easy setup for npm publishing using cargo-dist

echo "🚀 Setting up pipecheck for npm publishing..."

# Install cargo-dist if not already installed
if ! command -v cargo-dist &> /dev/null; then
    echo "📦 Installing cargo-dist..."
    cargo install cargo-dist
fi

# Initialize cargo-dist with npm support
echo "⚙️  Initializing cargo-dist..."
cargo dist init --installer=npm --yes

echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Update repository URL in Cargo.toml"
echo "2. Create a GitHub release to trigger builds"
echo "3. cargo-dist will automatically:"
echo "   - Build for all platforms"
echo "   - Create GitHub releases"
echo "   - Publish to npm"
echo ""
echo "To test locally:"
echo "  cargo dist build"
echo ""
echo "To publish:"
echo "  git tag v0.1.0"
echo "  git push --tags"
