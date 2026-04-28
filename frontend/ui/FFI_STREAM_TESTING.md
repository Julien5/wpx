# FFI Bridge Stream Testing Solution

## Problem

The Flutter app creates a stream via `backend.setSink()` which establishes a connection between Dart and Rust using flutter_rust_bridge. When tests complete, the stream cannot be properly closed, causing the test to hang.

## Root Cause

1. **Stream Creation**: `backend.setSink()` creates a `RustStreamSink` with an internal `ReceivePort`
2. **Rust Side Holds Reference**: The Rust backend stores the `StreamSink` in `self.sender`
3. **No Clean Closure**: flutter_rust_bridge (v2.11.1) doesn't provide a way to cleanly close streams from either side:
   - Canceling the Dart subscription doesn't close the stream controller
   - Dropping the Rust sink reference causes "Cannot close sink while adding stream" error
4. **Test Framework Waits**: Flutter's test framework waits for all async operations (including open ReceivePorts) to complete

## Why Closing Doesn't Work

Attempted solution that **failed**:
```rust
// In backend.rs
pub fn clear_sink(&mut self) {
    self.sender = std::sync::RwLock::new(None);  // Drops the sink
}
```

```dart
// In EventModel
void dispose() {
    _subscription?.cancel();
    backend.clearSink();  // ❌ Causes "Cannot close sink while adding stream"
    super.dispose();
}
```

**Error**: `Bad state: Cannot close sink while adding stream`

This happens because:
1. The Dart side has an active StreamController managing the bridge
2. When you drop the Rust sink, it tries to close the Dart StreamController
3. But the StreamController is still in "adding stream" state
4. Dart doesn't allow closing a StreamController in this state

## Solution: Conditional Stream Creation

Make the stream creation optional so tests can skip it:

### 1. Update EventModel (`lib/src/models/root.dart`)

```dart
class EventModel extends ChangeNotifier {
  final bridge.Bridge backend;
  late Stream<String> _stream;
  StreamSubscription<String>? _subscription; 
  String event = "";
  final bool _enableStream;

  EventModel({
    required this.backend, 
    bool enableStream = true,  // ✅ Add optional parameter
  }) : _enableStream = enableStream {
    if (_enableStream) {  // ✅ Only create stream if enabled
      _stream = backend.setSink();
      _subscription = _stream.listen((data) {  
        developer.log("EventModel.listen:$data");
        onEvent(data);
      });
    }
  }

  void onEvent(String data) {
    developer.log("onEvent:$data");
    event = data;
    notifyListeners();
  }

  String get() {
    return event;
  }

  @override
  void dispose() {
    developer.log("EventModel.dispose: cancelling subscription");
    _subscription?.cancel();  
    _subscription = null;
    super.dispose();
  }
}
```

### 2. Update Tests (`test/home_screen_golden_test.dart`)

```dart
// In the test, disable the stream
ChangeNotifierProvider(
  create: (_) => EventModel(
    backend: backend,
    enableStream: false,  // ✅ Disable stream in tests
  )
),
```

### 3. Production Code (No Changes Needed)

```dart
// In main.dart or ApplicationProvider
ChangeNotifierProvider(
  create: (_) => EventModel(backend: backend)  // enableStream defaults to true
),
```

## How It Works

1. **In Production**: `enableStream` defaults to `true`, stream is created normally
2. **In Tests**: Pass `enableStream: false` to skip stream creation entirely
3. **No FFI Bridge**: Without the stream, there's no ReceivePort to keep alive
4. **Test Completes**: The test framework sees no pending operations and finishes cleanly

## Benefits

- ✅ **No timeout needed**: Tests complete naturally
- ✅ **No hanging**: No ReceivePort or stream to wait for
- ✅ **Real backend**: Still tests with the actual Rust bridge for everything except events
- ✅ **Simple**: Just one parameter change in tests
- ✅ **Production unchanged**: Real app behavior is unaffected

## Testing

After this fix, tests should complete cleanly:

```bash
flutter test test/home_screen_golden_test.dart
```

No timeout wrapper needed!

## Files Modified

- `frontend/ui/lib/src/models/root.dart` - Added `enableStream` parameter to `EventModel`
- `frontend/ui/test/home_screen_golden_test.dart` - Pass `enableStream: false` in tests

## Alternative: Keep the Rust Methods

The `clear_sink()` methods added to the Rust code can be kept for future use, but they don't solve the test hanging issue due to flutter_rust_bridge's architecture. They might be useful for other cleanup scenarios in production code.

## Why Other Approaches Don't Work

### ❌ Drop Rust Reference
```rust
self.sender = None;  // Causes "Cannot close sink while adding stream"
```

### ❌ Cancel Subscription Only
```dart
_subscription?.cancel();  // Doesn't close ReceivePort
```

### ❌ Dispose Widget Tree
```dart
await tester.pumpWidget(Container());  // Stream still exists
```

### ✅ Don't Create Stream in Tests
```dart
enableStream: false  // Simple and effective!
```
