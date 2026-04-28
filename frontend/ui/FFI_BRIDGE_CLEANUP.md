# FFI Bridge Stream Cleanup Solution

## Problem

The Flutter app creates a stream via `backend.setSink()` which establishes a connection between Dart and Rust using flutter_rust_bridge. When the app exits or tests complete, the stream doesn't properly close, causing hangs.

## Root Cause

1. **Stream Creation**: `backend.setSink()` creates a `RustStreamSink` with an internal `ReceivePort`
2. **Rust Side Holds Reference**: The Rust backend stores the `StreamSink` in `self.sender`
3. **Canceling Subscription Isn't Enough**: When you call `subscription.cancel()` on the Dart side, it only stops listening to the stream but doesn't close the underlying stream controller
4. **Rust Reference Persists**: The Rust backend continues holding the sink, keeping the FFI bridge connection alive
5. **Test Framework Waits**: Flutter's test framework waits for all async operations to complete, causing infinite wait

## Solution

Added a `clear_sink()` method to explicitly tell the Rust side to drop its reference to the sink:

### 1. Rust Backend (`backend/src/backend.rs`)

```rust
/// Clear the event sink to allow proper cleanup and stream closure
pub fn clear_sink(&mut self) {
    self.sender = std::sync::RwLock::new(None);
}
```

### 2. Rust Bridge API (`rust/src/api/bridge.rs`)

```rust
/// Clear the event sink to allow proper cleanup
#[frb(sync)]
pub fn clear_sink(&mut self) {
    self.backend.clear_sink();
}
```

### 3. Dart EventModel (`lib/src/models/root.dart`)

```dart
@override
void dispose() {
  developer.log("EventModel.dispose: cancelling subscription");
  _subscription?.cancel();  
  _subscription = null;
  // Clear the sink on the Rust side to allow FFI bridge cleanup
  backend.clearSink();
  super.dispose();
}
```

## How It Works

1. **Subscription Cancel**: First cancels the Dart stream subscription to stop receiving events
2. **Clear Sink**: Calls `backend.clearSink()` which sets `self.sender = None` on the Rust side
3. **Drop Reference**: Dropping the Rust reference to the sink allows the flutter_rust_bridge to clean up
4. **Close Port**: The `ReceivePort` can now close properly
5. **Test Completes**: The test framework sees no more pending operations and allows the test to finish

## Testing

After this fix, the golden tests should complete without hanging. The timeout workaround in `run_golden_test.sh` should no longer be necessary, though it can remain as a safety measure.

To verify the fix works:
```bash
flutter test test/home_screen_golden_test.dart
```

The test should complete cleanly without requiring a timeout.

## Files Modified

- `backend/src/backend.rs` - Added `clear_sink()` method
- `frontend/ui/rust/src/api/bridge.rs` - Added FFI bridge wrapper for `clear_sink()`
- `frontend/ui/lib/src/models/root.dart` - Updated `EventModel.dispose()` to call `clearSink()`
- `frontend/ui/test/home_screen_golden_test.dart` - Added explicit widget disposal

## Note

After modifying the Rust code, you'll need to regenerate the Dart bindings:
```bash
flutter_rust_bridge_codegen generate
```

Or rebuild the project which will trigger the code generation automatically.
