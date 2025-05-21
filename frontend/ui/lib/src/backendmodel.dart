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
  final Future<String> future;
  final double currentEpsilon;
  String? _result;

  FutureRendering({required this.future, required this.currentEpsilon}) {
    future.then((value) => onCompleted(value));
  }

  void onCompleted(String value) {
    _result = value;
    notifyListeners();
  }

  bool done() {
    return _result != null;
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
      future: _frontend().renderSegmentWaypoints(segment: segment),
      currentEpsilon: _frontend().epsilon()
    );
  }

  FutureRendering renderSegmentTrack(FSegment segment) {
    return FutureRendering(
      future: _frontend().renderSegmentTrack(segment: segment),
      currentEpsilon: _frontend().epsilon()
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
