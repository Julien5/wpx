import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum TrackData { profile, yaxis, map, wheel }

class FutureRenderer with ChangeNotifier {
  final bridge.Segment _segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size? size;
  Set<bridge.InputType>? kinds;

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.trackData,
  }) : _segment = segment,
       _bridge = bridge {
    assert(_bridge.isLoaded());
  }

  void setProfileIndication(bridge.ProfileIndication p) {
    _bridge.setProfileIndication(p: p);
    reset();
  }

  void setKinds(Set<bridge.InputType> k) {
    kinds = k;
  }

  Set<bridge.InputType> getKinds() {
    if (kinds == null) {
      return bridge.allkinds();
    }
    return kinds!;
  }

  (int, int) getSizeAsTuple() {
    assert(size != null);
    int w = size!.width.floor();
    int h = w;
    if (size!.height.isFinite) {
      h = size!.height.floor();
    }
    return (w, h);
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
        segment: _segment,
        what: "profile",
        size: getSizeAsTuple(),
        kinds: getKinds(),
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "map",
        size: getSizeAsTuple(),
        kinds: getKinds(),
      );
    } else if (trackData == TrackData.yaxis) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "ylabels",
        size: getSizeAsTuple(),
        kinds: getKinds(),
      );
    } else if (trackData == TrackData.wheel) {
      log("[render-request-started:A]");
      assert(_bridge.isLoaded());
      log("[render-request-started:B]");
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "wheel",
        size: getSizeAsTuple(),
        kinds: getKinds(),
      );
      log("[render-request-started:C]");
    }
    notifyListeners();
    log("[render-request-started:$trackData]");
    _future!.then((value) => onCompleted(value));
  }

  int id() {
    return _segment.id();
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

class WheelRenderer extends FutureRenderer {
  WheelRenderer(
    bridge.Bridge bridge,
    bridge.Segment segment,
    Set<bridge.InputType> kinds,
  ) : super(bridge: bridge, segment: segment, trackData: TrackData.wheel) {
    super.setKinds(kinds);
  }
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

  void reset() {
    profileRenderer.reset();
    mapRenderer.reset();
    yaxisRenderer.reset();
  }
}
