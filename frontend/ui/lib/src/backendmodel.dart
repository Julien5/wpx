import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum TrackData { track, waypoints }

class FutureRenderer with ChangeNotifier {
  final bridge.Segment segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size size = Size(10, 10);

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
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
  TrackRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.track);
}

class WaypointsRenderer extends FutureRenderer {
  double visibility = 0;
  WaypointsRenderer(bridge.Bridge bridge, bridge.Segment segment)
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
  late bridge.Bridge _bridge;
  late String _filename;
  final List<Renderers> _segments = [];
  final List<bridge.Waypoint> _waypoints = [];

  SegmentsProvider() : _filename = "";

  static Future<SegmentsProvider> fromFilename(String filename) async {
    SegmentsProvider ret = SegmentsProvider();
    ret._filename = filename;
    developer.log("[filename] $filename");
    ret._bridge = await bridge.Bridge.create(filename: filename);
    ret._updateSegments();
    return ret;
  }

  static Future<SegmentsProvider> fromBytes(List<int> bytes) async {
    SegmentsProvider ret = SegmentsProvider();
    ret._bridge = await bridge.Bridge.fromContent(content: bytes);
    ret._updateSegments();
    return ret;
  }

  static Future<SegmentsProvider> demo() async {
    SegmentsProvider ret = SegmentsProvider();
    ret._bridge = await bridge.Bridge.initDemo();
    ret._updateSegments();
    return ret;
  }

  String filename() {
    return _filename;
  }

  @override
  void dispose() {
    developer.log("~SegmentsProvider");
    super.dispose();
  }

  void setContent(List<int> content) async {
    _bridge = await bridge.Bridge.fromContent(content: content);
    _updateSegments();
  }

  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
    _updateSegments();
  }

  Future<List<int>> generatePdf() {
    return _bridge.generatePdf();
  }

  Future<List<int>> generateGpx() {
    return _bridge.generateGpx();
  }

  void _updateSegments() {
    var segments = _bridge.segments();
    if (_segments.length != segments.length) {
      _segments.clear();
      for (var segment in segments) {
        var t = TrackRenderer(_bridge, segment);
        var w = WaypointsRenderer(_bridge, segment);
        _segments.add(Renderers(t, w));
      }
    } else {
      for (var renderers in _segments) {
        renderers.waypointsRendering.reset();
      }
    }
    _waypoints.clear();
    var wayPoints = _bridge.getWaypoints();
    for (var w in wayPoints) {
      _waypoints.add(w);
    }
    developer.log(
      "[SegmentsProvider] notifyListeners with ${_segments.length} segments",
    );
    notifyListeners();
  }

  List<Renderers> segments() {
    return _segments;
  }

  List<bridge.Waypoint> waypoints() {
    return _waypoints;
  }

  List<bridge.WaypointInfo> waypointInfos(bridge.Segment segment) {
    return _bridge.waypointsInfoTable(segment: segment);
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
  }

  String renderSegmentWaypointsSync(bridge.Segment segment, int w, int h) {
    return _bridge.renderSegmentWaypointsSync(segment: segment, w: w, h: h);
  }
}

class RootModel extends ChangeNotifier {
  List<SegmentsProvider> list = [];

  RootModel();

  @override
  void dispose() {
    developer.log("~RootModel");
    super.dispose();
  }

  Future<void> createSegmentsProvider(String filename) async {
    var ret = await SegmentsProvider.fromFilename(filename);
    list.add(ret);
    notifyListeners();
  }

  Future<void> createSegmentsProviderFromBytes(List<int> bytes) async {
    var ret = await SegmentsProvider.fromBytes(bytes);
    list.add(ret);
    notifyListeners();
  }

  Future<void> createSegmentsProviderForDemo() async {
    var ret = await SegmentsProvider.demo();
    list.add(ret);
    notifyListeners();
  }

  void unload() {
    list.clear();
    notifyListeners();
  }

  SegmentsProvider? provider() {
    if (list.isEmpty) {
      return null;
    }
    return list.first;
  }
}
