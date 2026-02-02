import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class EventModel extends ChangeNotifier {
  late Stream<String> _stream;
  String event = "";

  EventModel(bridge.Bridge bridge) {
    _stream = bridge.setSink();
    _stream.listen((data) {
      developer.log("EventModel.listen:$data");
      onEvent(data);
    });
  }

  void onEvent(String data) {
    developer.log("onEvent:$data");
    event = data;
    notifyListeners();
  }

  String get() {
    return event;
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
