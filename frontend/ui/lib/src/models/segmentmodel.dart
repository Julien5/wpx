import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  final bridge.Segment segment;
  final RootModel root;

  SegmentModel({required this.segment, required this.root}) {
    _bridge = root.getBridge();
    root.addListener(_onRootChanged);
  }

  @override
  void dispose() {
    root.removeListener(_onRootChanged);
    super.dispose();
  }

  void _onRootChanged() {
    notifyListeners();
  }

  void debug() {
    double length = _bridge.segmentStatistics(segment: segment).length / 1000;
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
    return FutureRenderer(
      bridge: _bridge,
      segment: segment,
      kinds: kinds,
      trackData: trackData,
    );
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

  bridge.SegmentStatistics statistics() {
    return _bridge.segmentStatistics(segment: segment);
  }

  List<bridge.Waypoint> allWaypoints() {
    return _bridge.getWaypoints(segment: segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> someWaypoints(Kinds kinds) {
    return _bridge.getWaypoints(segment: segment, kinds: kinds);
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
