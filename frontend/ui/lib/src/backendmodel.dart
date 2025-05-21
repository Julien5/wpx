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
  final FSegment segment;
  final TrackData trackData;
  final Frontend _frontend;
  Future<String>? future;
  String? _result;

  FutureRendering({
    required Frontend frontend,
    required this.segment,
    required this.trackData,
  }) : _frontend = frontend;

  void start() {
    if (trackData == TrackData.track) {
      developer.log("START track rendering for ${segment.id()}");
      future = _frontend.renderSegmentTrack(segment: segment);
    } else {
      developer.log("START waypoints rendering for ${segment.id()}");
      future = _frontend.renderSegmentWaypoints(segment: segment);
    }
    future!.then((value) => onCompleted(value));
    notifyListeners();
  }

  BigInt id() {
    return segment.id();
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
    return !started() && !done();
  }

  void onCompleted(String value) {
    _result = value;
    future = null;
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

class Segment extends InheritedWidget {
  final FutureRendering track;
  final FutureRendering waypoints;
  const Segment({
    super.key,
    required super.child,
    required this.track,
    required this.waypoints,
  });

  BigInt id() {
    return track.segment.id();
  }

  static Segment? maybeOf(BuildContext context) {
    return context.dependOnInheritedWidgetOfExactType<Segment>();
  }

  static Segment of(BuildContext context) {
    final Segment? ret = maybeOf(context);
    assert(ret != null);
    return ret!;
  }

  @override
  bool updateShouldNotify(covariant InheritedWidget oldWidget) {
    developer.log("UPDATE SHOULD NOTIFY");
    return true;
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

  Segment createRenderingsModel(FSegment segment, Widget child) {
    return Segment(
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
