import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/rust/api/frontend.dart';

class FrontendNotifier extends ChangeNotifier {
  final Frontend frontend;
  FrontendNotifier({required this.frontend});
  void notify() {
    notifyListeners();
  }
}

class BackendModel extends InheritedWidget {
  final FrontendNotifier notifier;
  //final String frontend;

  const BackendModel({super.key, required this.notifier, required super.child});

  Frontend frontend() {
    return notifier.frontend;
  }

  void incrementDelta() {
    frontend().changeParameter(eps: 10.0);
    notifier.notify();
  }

  void decrementDelta() {
    frontend().changeParameter(eps: -10.0);
    notifier.notify();
  }

  double epsilon() {
    return frontend().epsilon();
  }

  String value() {
    return "HI";
  }

  List<FSegment> segments() {
    return frontend().segments();
  }

  String renderSegmentWaypoints(FSegment segment) {
    return frontend().renderSegmentWaypointsSync(segment: segment);
  }

  Future<String> renderSegmentTrack(FSegment segment) {
    return frontend().renderSegmentTrack(segment: segment);
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
