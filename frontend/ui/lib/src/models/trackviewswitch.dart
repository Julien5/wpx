import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';

class TrackViewsSwitch extends ChangeNotifier {
  TrackData current = TrackData.wheel;
  final List<TrackData> exposed;
  TrackViewsSwitch({required this.exposed});

  static List<TrackData> wmp() {
    return [TrackData.wheel, TrackData.map, TrackData.profile];
  }

  void cycle() {
    if (current == TrackData.wheel) {
      return changeCurrent(TrackData.map);
    }
    if (current == TrackData.map) {
      return changeCurrent(TrackData.profile);
    }
    if (current == TrackData.profile) {
      return changeCurrent(TrackData.wheel);
    }
  }

  TrackData currentData() {
    return current;
  }

  void changeCurrent(TrackData d) {
    current = d;
    notifyListeners();
  }
}
