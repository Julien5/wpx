import 'dart:typed_data';

import 'package:flutter/widgets.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

class PendingContent {
  List<List<int>>? _bytes;

  static PendingContent makeFromBytes(List<List<int>> bytes) {
    var ret = PendingContent();
    ret._bytes = bytes;
    return ret;
  }

  List<Uint8List> contents() {
    return _bytes!.map((chunk) => Uint8List.fromList(chunk)).toList();
  }
}

class RootModel extends ChangeNotifier {
  final bridge.Bridge backend;
  PendingContent? _pendingContent;

  RootModel({required this.backend});

  bridge.Bridge getBackend() {
    return backend;
  }

  @override
  void notifyListeners() {
    debugPrint("ScreenConfiguration notifies");
    super.notifyListeners();
  }

  void setPendingContent(PendingContent u) {
    _pendingContent = u;
    backend.unload();
    notifyListeners();
  }

  Future<void> loadTrackFile(bridge.TrackFile f) async {
    _pendingContent = null;
    await backend.unload();
    await backend.loadTrackfile(trackfile: f);
    notifyListeners();
  }

  bool isLoaded() {
    return backend.isLoaded();
  }

  PendingContent? pendingContent() {
    return _pendingContent;
  }

  Future<List<bridge.TrackFile>> trackFiles() async {
    return await backend.trackfiles();
  }

  void notify() {
    notifyListeners();
  }
}
