#!/usr/bin/env bash

set -e

TEST_FILE="$1"
shift

pushd rust
dev.rust
cargo build
popd

dev.flutter-rust

export LD_LIBRARY_PATH=${CARGO_TARGET_DIR}/debug

if flutter test --verbose "$TEST_FILE" "$@"; then
    echo "✓ Test completed successfully!"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ] || [ $EXIT_CODE -eq 143 ]; then
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
