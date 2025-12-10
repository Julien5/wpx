import 'package:flutter/material.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class WaypointsModel with ChangeNotifier {
  final bridge.Bridge brd;
  bridge.Segment segment;
  WaypointsModel({required this.brd, required this.segment});

  List<bridge.Waypoint> all() {
    return brd.getWaypoints(segment: segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> some(Kinds kinds) {
    var ret = brd.getWaypoints(segment: segment, kinds: kinds);
    ret[0].info;
    return ret;
  }
}
