import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class EventModel extends ChangeNotifier {
  final bridge.Bridge backend;
  late Stream<String> _stream;
  String event = "";

  EventModel({required this.backend}) {
    _stream = backend.setSink();
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
  final bridge.Bridge backend;

  RootModel({required this.backend});

  bridge.Bridge getBackend() {
    return backend;
  }

  Future<void> loadDemo() async {
    developer.log("load demo");
    await backend.loadDemo();
  }

  Future<void> loadContent(List<int> bytes) async {
    developer.log("load ${bytes.length} bytes");
    await backend.loadContent(content: bytes);
  }

  Future<List<int>> generateGpx() {
    return backend.generateGpx();
  }

  Future<List<int>> generatePdf() {
    return backend.generatePdf();
  }

  Future<List<int>> generateZip() {
    return backend.generateZip();
  }

  bridge.SegmentStatistics statistics() {
    return backend.statistics();
  }

  List<bridge.Segment> segments() {
    return backend.segments();
  }

  void notify() {
    notifyListeners();
  }
}
