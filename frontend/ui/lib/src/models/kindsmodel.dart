import 'package:flutter/material.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';

class KindsModel with ChangeNotifier {
  Kinds kinds = [
    Kind.controls,
    Kind.gpxWaypoints,
    Kind.cutOff,
    Kind.cities,
    Kind.hamlets,
    Kind.mountains,
    Kind.villages,
  ];
  static final Kinds osmKinds = [
    Kind.cities,
    Kind.hamlets,
    Kind.mountains,
    Kind.villages,
  ];

  bool? osmIsLoaded;

  KindsModel();

  @override
  void notifyListeners() {
    debugPrint('=> KindsModel notifies');
    super.notifyListeners();
  }

  void addKind(Kind k) {
    if (kinds.contains(k)) {
      return;
    }
    kinds.add(k);
    notifyListeners();
  }

  bool? _hasControls;
  bool? _hasGPXWaypoints;
  void updateStatistics(SegmentStatistics s) {
    // we must notifiy only if there is a change, otherwise there is
    // an endless build loop (i dont known really what causes the loop).
    bool lhasControls = s.controls.isNotEmpty;
    bool lhasGPXWaypoints = s.waypoints.isNotEmpty;
    if (lhasControls == hasControls() &&
        lhasGPXWaypoints == hasGPXWaypoints()) {
      return;
    }
    _hasControls = lhasControls;
    _hasGPXWaypoints = lhasGPXWaypoints;
    notifyListeners();
  }

  bool hasControls() {
    return _hasControls != null && _hasControls!;
  }

  bool hasGPXWaypoints() {
    return _hasGPXWaypoints != null && _hasGPXWaypoints!;
  }

  void removeKind(Kind k) {
    if (!kinds.contains(k)) {
      return;
    }
    kinds.remove(k);
    notifyListeners();
  }

  void addOSM() {
    if (kinds.toSet().containsAll(osmKinds)) {
      return;
    }
    for (Kind k in {Kind.cities, Kind.hamlets, Kind.mountains, Kind.villages}) {
      kinds.add(k);
    }
    notifyListeners();
  }

  void removeOSM() {
    if (kinds.toSet().intersection(osmKinds.toSet()).isEmpty) {
      return;
    }
    for (Kind k in osmKinds) {
      kinds.remove(k);
    }
    notifyListeners();
  }
}
