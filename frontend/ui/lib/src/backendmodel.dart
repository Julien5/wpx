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

class BackendModel extends InheritedWidget {
  final BackendNotifier notifier;
  //final String frontend;

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

  String renderSegmentWaypoints(FSegment segment) {
    return _frontend().renderSegmentWaypointsSync(segment: segment);
  }

  Future<String> renderSegmentTrack(FSegment segment) {
    return _frontend().renderSegmentTrack(segment: segment);
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
