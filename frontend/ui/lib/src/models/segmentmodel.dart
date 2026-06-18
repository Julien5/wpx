import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

typedef Kinds = List<bridge.Kind>;

class SegmentModel extends ChangeNotifier {
  final bridge.Segment segment;
  final bridge.Bridge backend;

  SegmentModel({required this.segment, required this.backend});

  @override
  void notifyListeners() {
    debugPrint('=> SegmentModel ($segment) notifies');
    super.notifyListeners();
  }

  void debug() {
    double length = backend.segmentStatistics(segment: segment).length / 1000;
    developer.log("segment length:$length");
  }

  bridge.UserStepsOptions userStepsOptions() {
    return backend.getParameters().userStepsOptions;
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

  List<bridge.Waypoint> tableWaypoints() {
    return someWaypoints([bridge.Kind.gpxWaypoints, bridge.Kind.controls]);
  }

  void makeControlAtWaypoint(bridge.Waypoint waypoint, bool on) {
    backend.makeControlAtWaypoint(waypoint: waypoint, on_: on);
    notifyListeners();
  }

  void setControlTime(bridge.Waypoint waypoint, DateTime? time) {
    String? rfc3339time = time?.toUtc().toIso8601String();
    backend.setControlTime(waypoint: waypoint, time: rfc3339time);
    notifyListeners();
  }
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
    debugPrint("new users steps options:$p");
    notifyListeners();
  }

  Future<void> setParameters(bridge.Parameters p) async {
    await backend.setParameters(parameters: p);
    notifyListeners();
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

  void setProfileIndications(List<bridge.ProfileIndication> indications) {
    backend.setProfileIndications(indications: indications);
    notifyListeners();
  }
}
