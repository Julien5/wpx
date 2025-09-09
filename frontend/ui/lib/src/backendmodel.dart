import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum TrackData { profile,  map }

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
    if (trackData == TrackData.profile) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "profile",
        w: size.width.floor(),
        h: size.height.floor(),
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "map",
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

class ProfileRenderer extends FutureRenderer {
  ProfileRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.profile);

  void reset() {
    _future = null;
    _result = null;
    start();
  }
}

class MapRenderer extends FutureRenderer {
  MapRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.map);

  void reset() {
    _future = null;
    _result = null;
    start();
  }
}

class Renderers {
  final ProfileRenderer profileRendering;
  final MapRenderer mapRendering;
  Renderers(ProfileRenderer profile, MapRenderer map)
    : profileRendering = profile,
      mapRendering = map;
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
        var t = ProfileRenderer(_bridge, segment);
        var m = MapRenderer(_bridge, segment);
        _segments.add(Renderers(t, m));
      }
    }
    for (var renderers in _segments) {
      renderers.profileRendering.reset();
      renderers.mapRendering.reset();
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

  List<bridge.Waypoint> waypointTable(bridge.Segment segment) {
    return _bridge.waypointsTable(segment: segment);
  }

  String renderSegmentYAxis(bridge.Segment segment, Size size) {
    return _bridge.renderSegmentWhatSync(
      segment: segment,
      what: "ylabels",
      w: size.width.floor(),
      h: size.height.floor(),
    );
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
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
