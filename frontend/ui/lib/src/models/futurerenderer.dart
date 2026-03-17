import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/utils/utils.dart';

typedef TrackData = bridge.RenderFunction;

class FutureRenderer with ChangeNotifier {
  bridge.Segment _segment;
  final bridge.Bridge _backend;
  final List<TrackData> clients;

  final Set<bridge.Kind> kinds;

  Future<List<bridge.RenderOutput>>? _future;

  final Map<TrackData, Size?> _sizes = {};
  final Map<TrackData, RenderOutput> _results = {};

  bool _visible = false;
  bool _disposed = false;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.clients,
    required this.kinds,
  }) : _segment = segment,
       _backend = bridge {
    developer.log("[CREATE FUTURE RENDERER ($segment) ($clients) ($kinds)]");
    assert(_backend.isLoaded());
  }

  @override
  void dispose() {
    debugPrint("[renderer dispose ($clients)]");
    _future = null; // Clear the future reference
    if (_disposed) {
      return;
    }
    _disposed = true;
    super.dispose();
  }

  void updateSegment(bridge.Segment segment) {
    _segment = segment;
    reset();
  }

  Segment getSegment() {
    return _segment;
  }

  void addKind(Kind k) {
    if (kinds.contains(k)) {
      return;
    }
    kinds.add(k);
  }

  void removeKind(Kind k) {
    if (!kinds.contains(k)) {
      return;
    }
    kinds.remove(k);
  }

  Size getSize(TrackData d) {
    // this size is passed to the backend for rendering
    developer.log("wanted: $d has: ${_sizes.keys}");
    assert(_sizes.containsKey(d));
    assert(_sizes[d] != null);
    return _sizes[d]!;
  }

  bool isVisible() {
    return _visible;
  }

  void setVisible(bool b) {
    _visible = b;
    if (_visible) {
      start();
    }
  }

  void start() {
    debugPrint("[render-request] ($clients) ($kinds)");
    if (_disposed) {
      debugPrint("[render-request] ($clients) abort: renderer disposed");
      return;
    }
    if (_sizes.length != clients.length) {
      debugPrint(
        "[render-request] ($clients) abort: size is not set for all track data",
      );
      return;
    }
    if (!needsStart()) {
      debugPrint(
        "[render-request] ($clients) abort: renderer does not need start",
      );
      debugPrint(
        "[render-request] ($clients) abort: started=${started()} && done=${done()} && visible=${isVisible()};",
      );
      return;
    }

    log("[render-request-start:$clients]");
    _results.clear();
    List<bridge.RenderInput> renderInputs = [];
    for (TrackData d in clients) {
      (int, int) sizeParameter = sizeAsTuple(makeFinite(_sizes[d]!));
      renderInputs.add(
        bridge.RenderInput(kinds: kinds, function: d, size: sizeParameter),
      );
    }
    _future = _backend.renderSegment(segment: _segment, inputs: renderInputs);
    _future!.then((values) => onCompleted(values));
  }

  String id() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return "${clients.toString()}|${sortedKinds.join(",")}|${_segment.id()}";
  }

  bool started() {
    return _future != null;
  }

  bool needsStart() {
    return !started() && !done() && isVisible();
  }

  void onCompleted(List<bridge.RenderOutput> values) {
    if (_disposed) {
      developer.log("[renderer was disposed]");
      assert(_future == null);
      return;
    }

    for (bridge.RenderOutput output in values) {
      TrackData d = output.renderInput.function;
      Set<Kind> k = output.renderInput.kinds;

      debugPrint("found value for $d and $k");
      _results[d] = output;
    }

    _future = null;

    log("[render-request-comleted:${_results.keys}]");
    notifyListeners();
  }

  void reset() {
    _future = null;
    _results.clear();
    _sizes.clear();
    notifyListeners();
  }

  void restart() {
    _future = null;
    _results.clear();
    start();
  }

  bool setSize(TrackData d, Size newSize) {
    if (newSize == _sizes[d]) {
      return false;
    }
    debugPrint("setSize($d,$newSize)");
    _sizes[d] = newSize;
    _future = null;
    _results.clear();
    start();
    return true;
  }

  bool done() {
    return _results.length == clients.length;
  }

  String result(TrackData d) {
    assert(done());
    return _results[d]!.svg;
  }

  RenderOutput? renderOutput(TrackData d) {
    return _results[d];
  }
}
