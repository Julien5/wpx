import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  late bridge.Segment _segment;

  SegmentModel(bridge.Bridge bridge, bridge.Segment segment) {
    _bridge = bridge;
    _segment = segment;
  }

  bridge.Segment segment() {
    return _segment;
  }

  bridge.Bridge getBridge() {
    return _bridge;
  }

  bridge.UserStepsOptions userStepsOptions() {
    return _bridge.getUserStepOptions(segment: _segment);
  }

  void setUserStepsOptions(bridge.UserStepsOptions p) {
    _bridge.setUserStepOptions(segment: _segment, userStepsOptions: p);
    notifyListeners();
  }

  WheelRenderer createWheelRenderer(Kinds kinds) {
    return WheelRenderer(_bridge, _segment, kinds);
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.segmentStatistics(segment: _segment);
  }

  List<bridge.Waypoint> allWaypoints() {
    return _bridge.getWaypoints(segment: _segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> someWaypoints(Kinds kinds) {
    return _bridge.getWaypoints(segment: _segment, kinds: kinds);
  }

  void setUserStepGpxNameFormat(String format) {
    _bridge.setUserstepGpxNameFormat(format: format);
    notifyListeners();
  }

  void setControlGpxNameFormat(String format) {
    _bridge.setControlGpxNameFormat(format: format);
    notifyListeners();
  }
}
