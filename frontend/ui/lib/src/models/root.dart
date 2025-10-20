import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

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
  EventModel? _eventModel;

  RootModel() {
    _bridge = bridge.Bridge.make();
  }

  bridge.Bridge getBridge() {
    return _bridge;
  }

  EventModel eventModel() {
    _eventModel ??= EventModel(_bridge);
    return _eventModel!;
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
    notifyListeners();
  }

  Future<List<int>> generateGpx() {
    return _bridge.generateGpx();
  }

  Future<List<int>> generatePdf() {
    return _bridge.generatePdf();
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
  }

  List<bridge.Segment> segments() {
    return _bridge.segments();
  }
 
}
