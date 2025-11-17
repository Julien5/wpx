import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum TrackData { profile, yaxis, map }

class FutureRenderer with ChangeNotifier {
  bridge.Segment segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size? size;

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
    required this.segment,
    required this.trackData,
    
  }) : _bridge = bridge;

  void updateSegment(bridge.Segment newSegment) {
    segment=newSegment;
    reset();
  }

  (int,int) getSizeAsTuple() {
    return (size!.width.floor(),size!.height.floor());
  }

  void start() {
    if (size == null) {
      log("[render-request:$trackData] size is not set");
      return;
    }
    log("[render-request-start:$trackData]");
    _result = null;
    if (trackData == TrackData.profile) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "profile",
        size: getSizeAsTuple(),
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "map",
        size: getSizeAsTuple(),
      );
    } else if (trackData == TrackData.yaxis) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "ylabels",
        size: getSizeAsTuple(),
      );
    }
    notifyListeners();
    _future!.then((value) => onCompleted(value));
  }

  int id() {
    return segment.id();
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
    log("[render-request-comleted:$trackData]");
    notifyListeners();
  }

  void reset() {
    _future = null;
    _result = null;
    start();
  }

  bool setSize(Size newSize) {
    if (newSize == size) {
      return false;
    }
    size = newSize;
    _future = null;
    _result = null;
    Future.delayed(const Duration(milliseconds: 0), () {
      start();
    });
    return true;
  }

  bool done() {
    return _result != null;
  }

  String result() {
    assert(_result != null);
    return _result!;
  }
}

class ProfileRenderer extends FutureRenderer {
  ProfileRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.profile);
}

class YAxisRenderer extends FutureRenderer {
  YAxisRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.yaxis);
}

class MapRenderer extends FutureRenderer {
  MapRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.map);
}

class Renderers {
  final ProfileRenderer profileRenderer;
  final YAxisRenderer yaxisRenderer;
  final MapRenderer mapRenderer;
  Renderers(ProfileRenderer profile, YAxisRenderer yaxis, MapRenderer map)
    : profileRenderer = profile,
      yaxisRenderer = yaxis,
      mapRenderer = map;

  static Renderers make(bridge.Bridge bridge, bridge.Segment segment) {
    var t = ProfileRenderer(bridge, segment);
    var m = MapRenderer(bridge, segment);
    var y = YAxisRenderer(bridge, segment);
    return Renderers(t, y, m);
  }

  void updateSegment(bridge.Segment segment) {
    profileRenderer.updateSegment(segment);
    mapRenderer.updateSegment(segment);
    yaxisRenderer.updateSegment(segment);
  }

  void reset() {
    profileRenderer.reset();
    mapRenderer.reset();
    yaxisRenderer.reset();
  }
}
