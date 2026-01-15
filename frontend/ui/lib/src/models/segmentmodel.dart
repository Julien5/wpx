import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  late bridge.Segment _segment;

  SegmentModel(RootModel root, bridge.Segment segment) {
    _bridge = root.getBridge();
    _segment = segment;
  }

  bridge.Segment segment() {
    return _segment;
  }

  void debug() {
    double length = _bridge.segmentStatistics(segment: _segment).length / 1000;
    developer.log("segment length:$length");
  }

  bridge.UserStepsOptions userStepsOptions() {
    return _bridge.getParameters().userStepsOptions;
  }

  void setUserStepsOptions(bridge.UserStepsOptions p) {
    _bridge.setUserStepOptions(userStepsOptions: p);
    notify();
  }

  FutureRenderer makeRenderer(Kinds kinds, TrackData trackData) {
    if (trackData == TrackData.wheel) {
      return WheelRenderer(_bridge, _segment, kinds);
    }
    if (trackData == TrackData.profile) {
      return ProfileRenderer(_bridge, _segment, kinds);
    }
    if (trackData == TrackData.map) {
      return MapRenderer(_bridge, _segment, kinds);
    }
    throw Exception("invalid track data");
  }

  void notify() {
    notifyListeners();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
    notify();
  }

  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  ProfileRenderer createProfileRenderer(Kinds kinds) {
    return ProfileRenderer(_bridge, _segment, kinds);
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
