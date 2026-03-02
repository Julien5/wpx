import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/utils.dart';

typedef TrackData = bridge.RenderFunction;

class FutureRenderer with ChangeNotifier {
  bridge.Segment _segment;
  final bridge.Bridge _backend;
  final List<TrackData> trackData;

  final Set<bridge.Kind> kinds;

  Future<List<bridge.RenderOutput>>? _future;

  final Map<TrackData, Size?> _sizes = {};
  final Map<TrackData, String?> _results = {};
  bool _disposed = false;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.trackData,
    required this.kinds,
  }) : _segment = segment,
       _backend = bridge {
    developer.log("[CREATE FUTURE RENDERER ($segment) ($trackData)]");
    assert(_backend.isLoaded());
  }

  @override
  void dispose() {
    debugPrint("[renderer dispose ($trackData)]");
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

  Size getSize(TrackData d) {
    // this size is passed to the backend for rendering
    developer.log("wanted: $d has: ${_sizes.keys}");
    assert(_sizes.containsKey(d));
    assert(_sizes[d] != null);
    return _sizes[d]!;
  }

  void start() {
    debugPrint("start renderer!");
    if (_disposed) {
      debugPrint("attempt to start a disposed renderer!");
      return;
    }
    if (_sizes.length != trackData.length) {
      debugPrint("[render-request] size is not set for all track data");
      return;
    }
    double length = _backend.segmentStatistics(segment: _segment).length / 1000;
    log("[render-request-start:$trackData] [length:$length]");
    _results.clear();

    List<bridge.RenderInput> renderInputs = [];
    for (TrackData d in trackData) {
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
    return "${trackData.toString()}|${sortedKinds.join(",")}|${_segment.id()}";
  }

  bool started() {
    return _future != null;
  }

  bool needsStart() {
    return !started() && !done();
  }

  void onCompleted(List<bridge.RenderOutput> values) {
    if (_disposed) {
      developer.log("[renderer was disposed]");
      assert(_future == null);
      return;
    }

    for (bridge.RenderOutput value in values) {
      TrackData d = value.renderInput.function;
      debugPrint("found value for $d");
      _results[d] = value.svg;
    }

    _future = null;
    log("[render-request-comleted:${_results.keys}]");
    notifyListeners();
  }

  void reset() {
    _future = null;
    _results.clear();
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
    debugPrint("old size:${_sizes[d]} new size:$newSize");
    _sizes[d] = newSize;
    _future = null;
    _results.clear();
    return true;
  }

  bool done() {
    debugPrint("results:${_results.keys} vs $trackData");
    return _results.length == trackData.length;
  }

  String result(TrackData d) {
    assert(done());
    return _results[d]!;
  }
}
