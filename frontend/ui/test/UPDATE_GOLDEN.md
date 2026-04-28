# Updating Golden Files

## Issue

The golden test uses the real Rust bridge and initializes `EventModel`, which creates an infinite stream (`backend.setSink()`). This causes the test to hang during cleanup because Flutter's test framework waits for all async operations to complete.

## Workaround

Use `timeout` command to kill the test after the golden file is generated:

```bash
# Update goldens (will timeout but file is generated)
timeout --preserve-status 15 flutter test test/home_screen_golden_test.dart --update-goldens

# Verify goldens (will also timeout)
timeout --preserve-status 15 flutter test test/home_screen_golden_test.dart
```

The golden file is successfully generated/compared **before** the hang occurs, so the timeout doesn't affect the test result.

## Alternative: Mock EventModel

If the timeout is unacceptable, you can modify the test to use a mock EventModel that doesn't start the infinite stream:

```dart
class MockEventModel extends EventModel {
  MockEventModel({required super.backend}) {
    // Override constructor to not start the stream
  }
  
  @override
  Stream<String> get stream => const Stream.empty();
}
```

Then use `MockEventModel` instead of `EventModel` in the test providers.
