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
  double epsilon = 0;

  FutureRendering({
    required this.frontend,
    required this.segment,
    required this.trackData,
  });

  bool equal(FutureRendering other) {
    return epsilon == other.epsilon &&
        segment.id() == other.segment.id() &&
        trackData == other.trackData &&
        _result == other._result;
  }

  double currentEpsilon() {
    return epsilon;
  }

  void start() {
    epsilon = frontend.epsilon();
    if (trackData == TrackData.track) {
      developer.log("START track rendering for ${segment.id()}");
      future = frontend.renderSegmentTrack(segment: segment);
    } else {
      developer.log("START waypoints rendering for ${segment.id()}");
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

  bool needsStart() {
    return future == null && _result == null;
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

class RenderingsModel extends InheritedWidget {
  final FutureRendering track;
  final FutureRendering waypoints;
  const RenderingsModel({
    super.key,
    required super.child,
    required this.track,
    required this.waypoints,
  });

  @override
  bool updateShouldNotify(covariant InheritedWidget other) {
    debugPrint("updateShouldNotify");
    return true;
    //var eq = track.equal(other.track) && waypoints.equal(other.waypoints);
    //    return !eq;
  }

  static RenderingsModel? maybeOf(BuildContext context) {
    debugPrint("RenderingsModel maybeOf");
    return context.dependOnInheritedWidgetOfExactType<RenderingsModel>();
  }

  static RenderingsModel of(BuildContext context) {
    final RenderingsModel? ret = maybeOf(context);
    assert(ret != null);
    return ret!;
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

  FutureRendering _renderSegmentWaypoints(FSegment segment) {
    return FutureRendering(
      frontend: _frontend(),
      segment: segment,
      trackData: TrackData.waypoints,
    );
  }

  FutureRendering _renderSegmentTrack(FSegment segment) {
    return FutureRendering(
      frontend: _frontend(),
      segment: segment,
      trackData: TrackData.track,
    );
  }

  RenderingsModel createRenderingsModel(FSegment segment, Widget child) {
    return RenderingsModel(
      track: _renderSegmentTrack(segment),
      waypoints: _renderSegmentWaypoints(segment),
      child: child,
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
