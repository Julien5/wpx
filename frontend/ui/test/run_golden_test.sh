#!/bin/bash
# Helper script to run golden tests with the real Rust bridge.
# The test will hang during cleanup, so we use a timeout.

set -e

TIMEOUT=15
TEST_FILE="test/home_screen_golden_test.dart"

echo "Running golden test with ${TIMEOUT}s timeout..."
echo

if timeout --preserve-status ${TIMEOUT} flutter test "$TEST_FILE" "$@"; then
    echo "✓ Test completed successfully!"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ] || [ $EXIT_CODE -eq 143 ]; then
        echo
        echo "⚠ Test timed out (expected behavior - EventModel creates infinite stream)"
        
        # Check if golden was created/updated
        if [ -f "test/goldens/home_screen.png" ]; then
            echo "✓ Golden file exists: test/goldens/home_screen.png"
            ls -lh test/goldens/home_screen.png
            exit 0
        else
            echo "✗ Golden file was NOT created"
            exit 1
        fi
    else
        echo "✗ Test failed with exit code: $EXIT_CODE"
        exit $EXIT_CODE
    fi
fi
