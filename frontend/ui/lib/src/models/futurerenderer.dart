import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/utils.dart';

enum TrackData { profile, yaxis, map, wheel, pages }

class FutureRenderer with ChangeNotifier {
  bridge.Segment _segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size? size;
  final Set<bridge.InputType> kinds;

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.trackData,
    required this.kinds,
  }) : _segment = segment,
       _bridge = bridge {
    assert(_bridge.isLoaded());
  }

  void setProfileIndication(bridge.ProfileIndication p) {
    _bridge.setProfileIndication(p: p);
    restart();
  }

  @override
  void dispose() {
    developer.log("[renderer dispose]");
    _future = null; // Clear the future reference
    super.dispose();
  }

  void updateSegment(bridge.Segment segment) {
    _segment = segment;
    reset();
  }

  Size getSize() {
    // this size is passed to the backend for rendering
    assert(size != null);
    return size!;
  }

  void start() {
    if (size == null) {
      log("[render-request:$trackData] size is not set");
      return;
    }
    double length = _bridge.segmentStatistics(segment: _segment).length / 1000;
    log("[render-request-start:$trackData] [length:$length]");
    _result = null;
    (int, int) sizeParameter = sizeAsTuple(makeFinite(getSize()));
    if (trackData == TrackData.profile) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "profile",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "map",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.yaxis) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "ylabels",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.wheel) {
      log("[render-request-started:A]");
      assert(_bridge.isLoaded());
      log("[render-request-started:B]");
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "wheel",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.pages) {
      log("[render-request-started:A]");
      assert(_bridge.isLoaded());
      log("[render-request-started:B]");
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "wheel/pages",
        size: sizeParameter,
        kinds: kinds,
      );
      log("[render-request-started:C]");
    }
    log("[render-request-started:$trackData]");
    _future!.then((value) => onCompleted(value));
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

  void onCompleted(String value) {
    if (_future == null) {
      developer.log("[renderer was disposed?]");
      return;
    }
    _result = value;
    _future = null;
    log("[render-request-comleted:$trackData]");
    notifyListeners();
  }

  void reset() {
    _future = null;
    _result = null;
    notifyListeners();
  }

  void restart() {
    _future = null;
    _result = null;
    start();
  }

  bool setSize(Size newSize) {
    if (newSize == size) {
      return false;
    }
    size = newSize;
    _future = null;
    _result = null;
    return true;
  }

  bool done() {
    return _result != null;
  }

  String result() {
    assert(_result != null);
    return _result!;
  }
}
