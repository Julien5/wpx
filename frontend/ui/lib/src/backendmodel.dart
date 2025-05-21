import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/rust/api/frontend.dart';

class BackendNotifier extends ChangeNotifier {
  final Frontend frontend;
  BackendNotifier({required this.frontend});
  void notify() {
    notifyListeners();
  }
}

enum TrackData { track, waypoints }

class FutureRendering extends ChangeNotifier {
  FSegment segment;
  TrackData trackData;
  Frontend frontend;
  Future<String>? future;
  String? _result;

  FutureRendering({
    required this.frontend,
    required this.segment,
    required this.trackData,
  });

  double currentEpsilon() {
    return frontend.epsilon();
  }

  void start() {
    developer.log("START future");
    if (trackData == TrackData.track) {
      future = frontend.renderSegmentTrack(segment: segment);
    } else {
      future = frontend.renderSegmentWaypoints(segment: segment);
    }
    future!.then((value) => onCompleted(value));
    notifyListeners();
  }

  void reset() {
    future = null;
    _result = null;
    notifyListeners();
  }

  bool started() {
    return future != null;
  }

  void onCompleted(String value) {
    _result = value;
    future = null;
    notifyListeners();
  }

  bool done() {
    return _result != null;
  }

  bool running() {
    return future != null && _result == null;
  }

  String result() {
    assert(_result != null);
    return _result!;
  }
}

class BackendModel extends InheritedWidget {
  final BackendNotifier notifier;

  const BackendModel({super.key, required this.notifier, required super.child});

  Frontend _frontend() {
    return notifier.frontend;
  }

  void incrementDelta() {
    _frontend().changeParameter(eps: 10.0);
    notifier.notify();
  }

  void decrementDelta() {
    _frontend().changeParameter(eps: -10.0);
    notifier.notify();
  }

  double epsilon() {
    return _frontend().epsilon();
  }

  List<FSegment> segments() {
    return _frontend().segments();
  }

  String renderSegmentWaypointsSync(FSegment segment) {
    return _frontend().renderSegmentWaypointsSync(segment: segment);
  }

  FutureRendering renderSegmentWaypoints(FSegment segment) {
    return FutureRendering(
      frontend: _frontend(),
      segment: segment,
      trackData: TrackData.waypoints,
    );
  }

  FutureRendering renderSegmentTrack(FSegment segment) {
    return FutureRendering(
      frontend: _frontend(),
      segment: segment,
      trackData: TrackData.track,
    );
  }

  @override
  bool updateShouldNotify(covariant BackendModel oldWidget) {
    var ret = oldWidget.epsilon() != epsilon();
    developer.log("update=$ret");
    return ret;
  }

  static BackendModel? maybeOf(BuildContext context) {
    return context.dependOnInheritedWidgetOfExactType<BackendModel>();
  }

  static BackendModel of(BuildContext context) {
    final BackendModel? ret = maybeOf(context);
    assert(ret != null);
    return ret!;
  }
}
