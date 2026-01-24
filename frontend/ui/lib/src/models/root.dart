import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class EventModel extends ChangeNotifier {
  late Stream<String> stream;
  List<String> events = [];
  EventModel(bridge.Bridge bridge) {
    stream = bridge.setSink();
    stream.listen(onEvents);
  }
  void onEvents(String data) {
    developer.log("EventModel:$data");
    events.add(data);
    notifyListeners();
  }
}

class RootModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  EventModel? _eventModel;
  bridge.Segment? _trackSegment;

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
    developer.log("load demo");
    _trackSegment = null;
    await _bridge.loadDemo();
  }

  Future<void> loadContent(List<int> bytes) async {
    developer.log("load ${bytes.length} bytes");
    _trackSegment = null;
    await _bridge.loadContent(content: bytes);
  }

  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
    notifyListeners();
  }

  void setProfileIndication(bridge.ProfileIndication p) {
    _bridge.setProfileIndication(p: p);
    notifyListeners();
  }

  Future<List<int>> generateGpx() {
    return _bridge.generateGpx();
  }

  Future<List<int>> generatePdf() {
    return _bridge.generatePdf();
  }

  Future<List<int>> generateZip() {
    return _bridge.generateZip();
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
  }

  List<bridge.Segment> segments() {
    return _bridge.segments();
  }

  bridge.Segment trackSegment() {
    _trackSegment ??= _bridge.trackSegment();
    return _trackSegment!;
  }
}

class ParameterChanger {
  bridge.Parameters init;
  ParameterChanger({required this.init});
  bridge.Parameters current() {
    return init;
  }

  bridge.Parameters changeSpeed(double speed) {
    bridge.Parameters ret = bridge.Parameters(
      speed: speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeStartTime(DateTime time) {
    String rfc3339time = time.toIso8601String();
    if (!rfc3339time.endsWith("Z")) {
      rfc3339time = "${rfc3339time}Z";
    }
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: rfc3339time,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeSegmentLength(double length) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: length,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeSegmentOverlap(double overlap) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: overlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }
}
