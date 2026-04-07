#!/bin/bash
# Pre-commit hook for pipecheck

echo "🔍 Checking workflows with pipecheck..."

# Check if any workflow files changed
WORKFLOW_FILES=$(git diff --cached --name-only | grep -E '\.(github/workflows|gitlab-ci|circleci).*\.ya?ml$')

if [ -n "$WORKFLOW_FILES" ]; then
    if command -v pipecheck &> /dev/null; then
        pipecheck --all --strict
        if [ $? -ne 0 ]; then
            echo ""
            echo "❌ Workflow validation failed!"
            echo "Fix the errors above or use 'git commit --no-verify' to skip"
            exit 1
        fi
        echo "✅ All workflows valid!"
    else
        echo "⚠️  pipecheck not installed, skipping validation"
    fi
fi

exit 0
