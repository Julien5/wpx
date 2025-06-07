import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/rust/api/bridge.dart';

enum TrackData { track, waypoints }

class FutureRenderer with ChangeNotifier {
  final Segment segment;
  final TrackData trackData;
  final Bridge _bridge;

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required Bridge bridge,
    required this.segment,
    required this.trackData,
  }) : _bridge = bridge;

  void start() {
    _result = null;
    if (trackData == TrackData.track) {
      _future = _bridge.renderSegmentTrack(segment: segment);
    } else {
      _future = _bridge.renderSegmentWaypoints(segment: segment);
    }
    notifyListeners();
    _future!.then((value) => onCompleted(value));
  }

  BigInt id() {
    return segment.id;
  }

  bool started() {
    return _future != null;
  }

  bool needsStart() {
    return !started() && !done();
  }

  void onCompleted(String value) {
    _result = value;
    _future = null;
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

class TrackRenderer extends FutureRenderer {
  TrackRenderer(Bridge bridge, Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.track);
}

class WaypointsRenderer extends FutureRenderer {
  double visibility = 0;
  WaypointsRenderer(Bridge bridge, Segment segment)
    : super(
        bridge: bridge,
        segment: segment,
        trackData: TrackData.waypoints,
      );

  void updateVisibility(double v) {
    visibility = v;
    _update();
  }

  void reset() {
    _future = null;
    _result = null;
    _update();
  }

  void _update() {
    if (visibility < 0.5) {
      return;
    }
    if (needsStart()) {
      start();
    }
  }
}

class Renderers {
  final TrackRenderer trackRendering;
  final WaypointsRenderer waypointsRendering;
  Renderers(TrackRenderer track, WaypointsRenderer waypoints)
    : trackRendering = track,
      waypointsRendering = waypoints;
}

class RenderingsProvider extends MultiProvider {
  final Renderers renderers;

  RenderingsProvider(
    Renderers r,
    Widget child, {
    super.key,
  }) : renderers = r,
       super(
         providers: [
           ChangeNotifierProvider.value(value: r.trackRendering),
           ChangeNotifierProvider.value(value: r.waypointsRendering),
         ],
         child: child,
       );
}

class SegmentsProvider extends ChangeNotifier {
  Bridge? _bridge;
  final List<Renderers> _segments = [];

  SegmentsProvider(Bridge f) {
    _bridge = f;
    _updateSegments();
  }

  void incrementDelta() {
    _bridge!.changeParameter(eps: 10.0);
    _updateSegments();
  }

  void decrementDelta() {
    _bridge!.changeParameter(eps: -10.0);
    _updateSegments();
  }

  void _updateSegments() {
    var segments = _bridge!.segments();
    assert(_segments.isEmpty || _segments.length == segments.length);
    if (_segments.isEmpty) {
      for(var segment in segments) {
        var t=TrackRenderer(_bridge!,segment);
        var w=WaypointsRenderer(_bridge!,segment);
        _segments.add(Renderers(t,w));
      }
    } else {
      for(var renderers in _segments) {
        renderers.waypointsRendering.reset();
      }
    }
    notifyListeners();
  }

  List<Renderers> segments() {
    return _segments;
  }

  String renderSegmentWaypointsSync(Segment segment) {
    return _bridge!.renderSegmentWaypointsSync(segment: segment);
  }
}
