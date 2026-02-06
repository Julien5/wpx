import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  final bridge.Segment segment;
  final bridge.Bridge root;

  SegmentModel({required this.segment, required this.root});

  void debug() {
    double length = root.segmentStatistics(segment: segment).length / 1000;
    developer.log("segment length:$length");
  }

  bridge.UserStepsOptions userStepsOptions() {
    return root.getParameters().userStepsOptions;
  }

  void setUserStepsOptions(bridge.UserStepsOptions p) {
    root.setUserStepOptions(userStepsOptions: p);
    notify();
  }

  FutureRenderer makeRenderer(Kinds kinds, TrackData trackData) {
    return FutureRenderer(
      bridge: root,
      segment: segment,
      kinds: kinds,
      trackData: trackData,
    );
  }

  void notify() {
    notifyListeners();
  }

  void setParameters(bridge.Parameters p) {
    root.setParameters(parameters: p);
    notify();
  }

  bridge.Parameters parameters() {
    return root.getParameters();
  }

  bridge.SegmentStatistics statistics() {
    return root.segmentStatistics(segment: segment);
  }

  List<bridge.Waypoint> allWaypoints() {
    return root.getWaypoints(segment: segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> someWaypoints(Kinds kinds) {
    return root.getWaypoints(segment: segment, kinds: kinds);
  }

  void setUserStepGpxNameFormat(String format) {
    root.setUserstepGpxNameFormat(format: format);
    notifyListeners();
  }

  void setControlGpxNameFormat(String format) {
    root.setControlGpxNameFormat(format: format);
    notifyListeners();
  }
}
