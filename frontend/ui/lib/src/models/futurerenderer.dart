import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/utils.dart';

enum TrackData { profile, yaxis, map, wheel }

class FutureRenderer with ChangeNotifier {
  final bridge.Segment _segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size? size;
  final Set<bridge.InputType> kinds;

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.trackData,
    required this.kinds,
  }) : _segment = segment,
       _bridge = bridge {
    assert(_bridge.isLoaded());
  }

  void setProfileIndication(bridge.ProfileIndication p) {
    _bridge.setProfileIndication(p: p);
    restart();
  }

  Size getSize() {
    // this size is passed to the backend for rendering
    assert(size != null);
    return size!;
  }

  void start() {
    if (size == null) {
      log("[render-request:$trackData] size is not set");
      return;
    }
    log("[render-request-start:$trackData]");
    _result = null;
    (int, int) sizeParameter = sizeAsTuple(makeFinite(getSize()));
    if (trackData == TrackData.profile) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "profile",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "map",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.yaxis) {
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "ylabels",
        size: sizeParameter,
        kinds: kinds,
      );
    } else if (trackData == TrackData.wheel) {
      log("[render-request-started:A]");
      assert(_bridge.isLoaded());
      log("[render-request-started:B]");
      _future = _bridge.renderSegmentWhat(
        segment: _segment,
        what: "wheel",
        size: sizeParameter,
        kinds: kinds,
      );
      log("[render-request-started:C]");
    }
    notifyListeners();
    log("[render-request-started:$trackData]");
    _future!.then((value) => onCompleted(value));
  }

  String id() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return "${trackData.toString()}|${sortedKinds.join(",")}|${_segment.id()}";
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
    notifyListeners();
  }

  void restart() {
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
  ProfileRenderer(bridge.Bridge bridge, bridge.Segment segment, Kinds kinds)
    : super(
        bridge: bridge,
        segment: segment,
        trackData: TrackData.profile,
        kinds: kinds,
      );
}

class YAxisRenderer extends FutureRenderer {
  YAxisRenderer(bridge.Bridge bridge, bridge.Segment segment, Kinds kinds)
    : super(
        bridge: bridge,
        segment: segment,
        trackData: TrackData.yaxis,
        kinds: kinds,
      );
}

class MapRenderer extends FutureRenderer {
  MapRenderer(bridge.Bridge bridge, bridge.Segment segment, Kinds kinds)
    : super(
        bridge: bridge,
        segment: segment,
        trackData: TrackData.map,
        kinds: kinds,
      );
}

class WheelRenderer extends FutureRenderer {
  WheelRenderer(
    bridge.Bridge bridge,
    bridge.Segment segment,
    Set<bridge.InputType> kinds,
  ) : super(
        bridge: bridge,
        segment: segment,
        trackData: TrackData.wheel,
        kinds: kinds,
      );
}

class Renderers {
  final ProfileRenderer profileRenderer;
  final YAxisRenderer yaxisRenderer;
  final MapRenderer mapRenderer;
  Renderers(ProfileRenderer profile, YAxisRenderer yaxis, MapRenderer map)
    : profileRenderer = profile,
      yaxisRenderer = yaxis,
      mapRenderer = map;

  static Renderers make(
    bridge.Bridge bridge,
    bridge.Segment segment,
    Kinds kinds,
  ) {
    var t = ProfileRenderer(bridge, segment, kinds);
    var m = MapRenderer(bridge, segment, kinds);
    var y = YAxisRenderer(bridge, segment, kinds);
    return Renderers(t, y, m);
  }

  void reset() {
    profileRenderer.restart();
    mapRenderer.restart();
    yaxisRenderer.restart();
  }
}
