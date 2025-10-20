import 'package:flutter/material.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class WaypointsTableData with ChangeNotifier {
  final bridge.Bridge brd;
  bridge.Segment segment;
  WaypointsTableData({required this.brd, required this.segment});
  List<bridge.Waypoint> tableWaypoints() {
    return brd.waypointsTable(segment: segment);
  }
}