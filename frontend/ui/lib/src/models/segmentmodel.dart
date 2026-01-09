import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

typedef Kinds = Set<bridge.InputType>;

class SegmentModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  late bridge.Segment _segment;
  late Map<(TrackData, Kinds), FutureRenderer> _renderers;

  SegmentModel(bridge.Bridge bridge, bridge.Segment segment) {
    _bridge = bridge;
    _segment = segment;
    _renderers = <(TrackData, Kinds), FutureRenderer>{};
  }

  SegmentModel copy() {
    return SegmentModel(_bridge, _segment);
  }

  bridge.UserStepsOptions userStepsOptions() {
    return _bridge.getParameters().userStepsOptions;
  }

  void setUserStepsOptions(bridge.UserStepsOptions p) {
    _bridge.setUserStepOptions(userStepsOptions: p);
    notify();
  }

  FutureRenderer giveRenderer(Kinds kinds, TrackData trackData) {
    var key = (trackData, kinds);
    if (!_renderers.containsKey(key)) {
      if (trackData == TrackData.wheel) {
        _renderers[key] = WheelRenderer(_bridge, _segment, kinds);
      }
      if (trackData == TrackData.profile) {
        _renderers[key] = ProfileRenderer(_bridge, _segment, kinds);
      }
      if (trackData == TrackData.map) {
        _renderers[key] = MapRenderer(_bridge, _segment, kinds);
      }
    }
    return _renderers[key]!;
  }

  void resetRenderers() {
    for (final renderer in _renderers.values) {
      try {
        developer.log("(reset renderer) ${renderer.trackData}");
        renderer.reset();
      } catch (e, st) {
        developer.log('resetRenderers error: $e\n$st');
      }
    }
  }

  void notify() {
    resetRenderers();
    notifyListeners();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
    notify();
  }

  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  ProfileRenderer createProfileRenderer(Kinds kinds) {
    return ProfileRenderer(_bridge, _segment, kinds);
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.segmentStatistics(segment: _segment);
  }

  List<bridge.Waypoint> allWaypoints() {
    return _bridge.getWaypoints(segment: _segment, kinds: bridge.allkinds());
  }

  List<bridge.Waypoint> someWaypoints(Kinds kinds) {
    return _bridge.getWaypoints(segment: _segment, kinds: kinds);
  }

  void setUserStepGpxNameFormat(String format) {
    _bridge.setUserstepGpxNameFormat(format: format);
    notifyListeners();
  }

  void setControlGpxNameFormat(String format) {
    _bridge.setControlGpxNameFormat(format: format);
    notifyListeners();
  }
}
