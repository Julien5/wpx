import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum TrackData { profile, yaxis, map }

class FutureRenderer with ChangeNotifier {
  final bridge.Segment segment;
  final TrackData trackData;
  final bridge.Bridge _bridge;
  Size size = Size(10, 10);

  Future<String>? _future;
  String? _result;

  FutureRenderer({
    required bridge.Bridge bridge,
    required this.segment,
    required this.trackData,
  }) : _bridge = bridge;

  void start() {
    _result = null;
    if (trackData == TrackData.profile) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "profile",
        w: size.width.floor(),
        h: size.height.floor(),
      );
    } else if (trackData == TrackData.map) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "map",
        w: size.width.floor(),
        h: size.height.floor(),
      );
    } else if (trackData == TrackData.yaxis) {
      _future = _bridge.renderSegmentWhat(
        segment: segment,
        what: "ylabels",
        w: size.width.floor(),
        h: size.height.floor(),
      );
    }
    notifyListeners();
    _future!.then((value) => onCompleted(value));
  }

  BigInt id() {
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
    notifyListeners();
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

  void reset() {
    _future = null;
    _result = null;
    start();
  }
}

class YAxisRenderer extends FutureRenderer {
  YAxisRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.yaxis);

  void reset() {
    _future = null;
    _result = null;
    start();
  }
}

class MapRenderer extends FutureRenderer {
  MapRenderer(bridge.Bridge bridge, bridge.Segment segment)
    : super(bridge: bridge, segment: segment, trackData: TrackData.map);

  void reset() {
    _future = null;
    _result = null;
    start();
  }
}

class Renderers {
  final ProfileRenderer profileRendering;
  final YAxisRenderer yaxisRendering;
  final MapRenderer mapRendering;
  Renderers(ProfileRenderer profile, YAxisRenderer yaxis, MapRenderer map)
    : profileRendering = profile,
      yaxisRendering = yaxis,
      mapRendering = map;
}
