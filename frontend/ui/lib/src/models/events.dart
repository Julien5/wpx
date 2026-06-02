import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import 'package:wpx/src/rust/api/bridge.dart' as bridge;

class EventModel extends ChangeNotifier {
  final bridge.Bridge backend;
  final OsmStatus osmStatus = OsmStatus();
  late Stream<String> _stream;
  String event = "";

  EventModel({required this.backend}) {
    _stream = backend.setSink();
    _stream.listen((data) {
      debugPrint("EventModel.listen:$data");
      onEvent(data);
    });
  }

  void onEvent(String data) {
    debugPrint("onEvent:$data");
    event = data;
    if (data.startsWith("osm:")) {
      osmStatus.update(data);
    }
    if (data.startsWith("gpx:read")) {
      osmStatus.reset();
    }
    notifyListeners();
  }

  String get() {
    return event;
  }

  OsmStatus getOsmEvent() {
    return osmStatus;
  }
}

class OsmStatus {
  String _taskName = "";
  int current = -1;
  int total = -1;

  int downloadCurrent = -1;
  int downloadTotal = -1;
  int retry = -1;

  OsmStatus();

  void reset() {
    _taskName = "";
    current = -1;
    total = -1;
    downloadCurrent = -1;
    downloadTotal = -1;
  }

  String niceTaskName() {
    if (_taskName.contains("download")) {
      return "download";
    }
    if (_taskName.contains("read-cache")) {
      return "read cache";
    }
    if (_taskName.contains("write-cache")) {
      return "write cache";
    }
    if (_taskName.contains("wait-for-response")) {
      return "waiting for response";
    }
    if (_taskName.contains("retry")) {
      if (retry >= 0) {
        return "retry #$retry";
      }
      return "retry..";
    }
    if (_taskName.contains("sort")) {
      return "sort";
    }
    if (_taskName.contains("done")) {
      return "done";
    }
    return "...";
  }

  void update(String event) {
    List<String> parts = event.split(":");
    if (parts[0] != "osm") {
      return;
    }
    if (parts.length < 2) {
      return;
    }

    debugPrint("onEvent parts: $parts ${parts[0]} ${parts[1]}");
    _taskName = parts[1];
    if (_taskName.contains("progress")) {
      current = int.parse(parts[2]) + 1;
      total = int.parse(parts[3]);
      if (_taskName.contains("download")) {
        downloadCurrent = current - 1;
        downloadTotal = total;
      }
    }
    if (_taskName.contains("wait")) {
      current = downloadCurrent;
      total = downloadTotal;
    }

    if (_taskName.contains("retry")) {
      retry = int.parse(parts[2]) + 1;
    }
    debugPrint("onEvent current=$current total=$total");
  }

  bool done() {
    return _taskName.contains("done");
  }
}
