import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class SegmentData {
  Renderers renderers;
  List<bridge.Waypoint> tableWaypoints;
  SegmentData({required this.renderers, required this.tableWaypoints});
}

class EventModel extends ChangeNotifier {
  late Stream<String> stream;
  EventModel(bridge.Bridge bridge) {
    stream = bridge.setSink();
    //events.listen(onEvents);
  }
  /*
  void onEvents(String data) {
    developer.log("event:$data");
    notifyListeners();
  }
  */
}

class RootModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  final Map<bridge.Segment, SegmentData> _segments = {};
  EventModel? _eventModel;

  RootModel() {
    _bridge = bridge.Bridge.make();
  }

  EventModel eventModel() {
    _eventModel ??= EventModel(_bridge);
    return _eventModel!;
  }

  @override
  void dispose() {
    developer.log("~RootModel");
    super.dispose();
  }

  Future<void> loadDemo() async {
    await _bridge.loadDemo();
  }

  Future<void> loadContent(List<int> bytes) async {
    developer.log("load ${bytes.length} bytes");
    await _bridge.loadContent(content: bytes);
  }

  Future<void> loadFilename(String filename) async {
    developer.log("load $filename");
    await _bridge.loadFilename(filename: filename);
  }

  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
  }

  Map<bridge.Segment, SegmentData> segments() {
    return _segments;
  }

  void updateSegments() {
    _segments.clear();
    var segments = _bridge.segments();

    for (var segment in segments) {
      var t = ProfileRenderer(_bridge, segment);
      var m = MapRenderer(_bridge, segment);
      var y = YAxisRenderer(_bridge, segment);
      var W = _bridge.waypointsTable(segment: segment);
      _segments[segment] = SegmentData(
        renderers: Renderers(t, y, m),
        tableWaypoints: W,
      );
    }
  }
}
