import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/rust/api/bridge.dart';

enum TrackData { track, waypoints }

class FutureRenderer with ChangeNotifier {
  final Segment segment;
  final TrackData trackData;
  final Bridge _bridge;
  Size size = Size(10, 10);

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required Bridge bridge,
    required this.segment,
    required this.trackData,
  }) : _bridge = bridge;

  void start() {
    _result = null;
    if (trackData == TrackData.track) {
      _future = _bridge.renderSegmentTrack(
        segment: segment,
        w: size.width.floor(),
        h: size.height.floor(),
      );
    } else {
      _future = _bridge.renderSegmentWaypoints(
        segment: segment,
        w: size.width.floor(),
        h: size.height.floor(),
      );
    }
    notifyListeners();
    _future!.then((value) => onCompleted(value));
  }

  BigInt id() {
    return segment.id();
  }

  bool started() {
    return _future != null;
  }

  bool needsStart() {
    return !started() && !done();
  }

  void onCompleted(String value) {
    _result = value;
    _future = null;
    notifyListeners();
  }

  bool setSize(Size newSize) {
    if (newSize == size) {
      return false;
    }
    size = newSize;
    _future = null;
    _result = null;
    Future.delayed(const Duration(milliseconds: 0), () {
      start();
    });
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

class TrackRenderer extends FutureRenderer {
  TrackRenderer(Bridge bridge, Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.track);
}

class WaypointsRenderer extends FutureRenderer {
  double visibility = 0;
  WaypointsRenderer(Bridge bridge, Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.waypoints);

  void updateVisibility(double v) {
    visibility = v;
    _update();
  }

  void reset() {
    _future = null;
    _result = null;
    _update();
  }

  void _update() {
    if (visibility < 0.5) {
      return;
    }
    if (needsStart()) {
      start();
    }
  }
}

class Renderers {
  final TrackRenderer trackRendering;
  final WaypointsRenderer waypointsRendering;
  Renderers(TrackRenderer track, WaypointsRenderer waypoints)
    : trackRendering = track,
      waypointsRendering = waypoints;
}

class SegmentsProvider extends ChangeNotifier {
  Bridge? _bridge;
  final List<Renderers> _segments = [];
  final List<WayPoint> _waypoints = [];

  SegmentsProvider();

  void setFilename(String filename) async {
    developer.log("filename=$filename");
    _bridge = await Bridge.create(filename: filename);
    _updateSegments();
  }

  void setDemoContent() async {
    _bridge = await Bridge.initDemo();
    _updateSegments();
  }

  bool bridgeIsLoaded() {
    return _bridge != null;
  }

  void unload() {
    _bridge = null;
    _updateSegments();
  }

  void setContent(List<int> content) async {
    _bridge = await Bridge.fromContent(content: content);
    _updateSegments();
  }

  void incrementDelta() async {
    assert(_bridge != null);
    await _bridge!.adjustEpsilon(eps: 10.0);
    _updateSegments();
  }

  void decrementDelta() async {
    assert(_bridge != null);
    await _bridge!.adjustEpsilon(eps: -10.0);
    _updateSegments();
  }

  void _updateSegments() {
    if (_bridge == null) {
      _segments.clear();
      _waypoints.clear();
      notifyListeners();
      return;
    }
    assert(_bridge != null);
    var segments = _bridge!.segments();
    if (_segments.length != segments.length) {
      for (var segment in segments) {
        var t = TrackRenderer(_bridge!, segment);
        var w = WaypointsRenderer(_bridge!, segment);
        _segments.add(Renderers(t, w));
      }
    } else {
      for (var renderers in _segments) {
        renderers.waypointsRendering.reset();
      }
    }
    _waypoints.clear();
    var WP = _bridge!.getWayPoints();
    for (var w in WP) {
      _waypoints.add(w);
    }
    notifyListeners();
  }

  List<Renderers> segments() {
    return _segments;
  }

  List<WayPoint> waypoints() {
    return _waypoints;
  }

  String renderSegmentWaypointsSync(Segment segment, int w, int h) {
    return _bridge!.renderSegmentWaypointsSync(segment: segment, w: w, h: h);
  }
}
