import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  final bridge.Segment segment;
  final bridge.Bridge backend;

  SegmentModel({required this.segment, required this.backend});

  void debug() {
    double length = backend.segmentStatistics(segment: segment).length / 1000;
    developer.log("segment length:$length");
  }

  bridge.UserStepsOptions userStepsOptions() {
    return backend.getParameters().userStepsOptions;
  }

  FutureRenderer makeRenderer(Kinds kinds, TrackData trackData) {
    return FutureRenderer(
      bridge: backend,
      segment: segment,
      kinds: kinds,
      trackData: trackData,
    );
  }

  bridge.SegmentStatistics statistics() {
    return backend.segmentStatistics(segment: segment);
  }

  List<bridge.Waypoint> allWaypoints() {
    return backend.getWaypoints(segment: segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> someWaypoints(Kinds kinds) {
    return backend.getWaypoints(segment: segment, kinds: kinds);
  }
}

class TrackModel extends SegmentModel {
  TrackModel({required super.backend}) : super(segment: backend.trackSegment());
}

class ParameterModel extends ChangeNotifier {
  final bridge.Bridge backend;

  ParameterModel({required this.backend});

  void debug() {}

  bridge.UserStepsOptions userStepsOptions() {
    return backend.getParameters().userStepsOptions;
  }

  void setUserStepsOptions(bridge.UserStepsOptions p) {
    backend.setUserStepOptions(userStepsOptions: p);
    notify();
  }

  void notify() {
    notifyListeners();
  }

  void setParameters(bridge.Parameters p) {
    backend.setParameters(parameters: p);
    notify();
  }

  bridge.Parameters parameters() {
    return backend.getParameters();
  }

  void setUserStepGpxNameFormat(String format) {
    backend.setUserstepGpxNameFormat(format: format);
    notifyListeners();
  }

  void setControlGpxNameFormat(String format) {
    backend.setControlGpxNameFormat(format: format);
    notifyListeners();
  }

  void setProfileIndication(bridge.ProfileIndication p) {
    backend.setProfileIndication(p: p);
    notifyListeners();
  }
}
