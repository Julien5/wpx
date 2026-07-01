import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/utils/utils.dart';

typedef Kinds = List<bridge.Kind>;

class SegmentModel extends ChangeNotifier {
  final bridge.Segment segment;
  final bridge.Bridge backend;
  final bridge.TrackFile? trackFile;

  SegmentModel({
    required this.segment,
    required this.backend,
    required this.trackFile,
  });

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

  void makeControlAtWaypoint(bridge.Waypoint waypoint, bool on) async {
    backend.makeControlAtWaypoint(waypoint: waypoint, on_: on);
    if (trackFile != null) {
      backend.saveSmallParameters(trackfile: trackFile!);
    }
    notifyListeners();
  }

  void setControlTime(bridge.Waypoint waypoint, DateTime? time) async {
    String? rfc3339time = time?.toUtc().toIso8601String();
    backend.setControlTime(waypoint: waypoint, time: rfc3339time);
    if (trackFile != null) {
      backend.saveSmallParameters(trackfile: trackFile!);
    }
    notifyListeners();
  }

  void setUserStepsOptions(bridge.UserStepsOptions options) {
    ParameterChanger changer = ParameterChanger(init: parameters());
    changer.changeUserStepsOptions(options);
    setParameters(changer.current());
    debugPrint("new users steps options:$options");
    notifyListeners();
  }

  Future<void> saveTrackFile() async {
    if (trackFile != null) {
      await backend.saveGpxdata(trackfile: trackFile!);
      await backend.saveSmallParameters(trackfile: trackFile!);
    }
  }

  void setParameters(bridge.Parameters p) async {
    await backend.setParameters(parameters: p);
    if (trackFile != null) {
      await backend.saveSmallParameters(trackfile: trackFile!);
    }
    notifyListeners();
  }

  bridge.Parameters parameters() {
    return backend.getParameters();
  }
}
