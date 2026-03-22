import 'package:flutter/material.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class KindsModel with ChangeNotifier {
  Kinds kinds = {
    Kind.controls,
    Kind.gpxWaypoints,
    Kind.userStep,
    Kind.cities,
    Kind.hamlets,
    Kind.mountains,
    Kind.villages,
  };
  static final Kinds osmKinds = {
    Kind.cities,
    Kind.hamlets,
    Kind.mountains,
    Kind.villages,
  };

  bool? osmIsLoaded;
  SegmentStatistics? statistics;

  KindsModel();

  void addKind(Kind k) {
    if (kinds.contains(k)) {
      return;
    }
    kinds.add(k);
    notifyListeners();
  }

  void removeKind(Kind k) {
    if (!kinds.contains(k)) {
      return;
    }
    kinds.remove(k);
    notifyListeners();
  }

  void addOSM() {
    if (kinds.containsAll(osmKinds)) {
      return;
    }
    for (Kind k in {Kind.cities, Kind.hamlets, Kind.mountains, Kind.villages}) {
      kinds.add(k);
    }
    notifyListeners();
  }

  void removeOSM() {
    if (kinds.intersection(osmKinds).isEmpty) {
      return;
    }
    for (Kind k in osmKinds) {
      kinds.remove(k);
    }
    notifyListeners();
  }
}
