import 'package:flutter/material.dart';
import 'package:ui/src/models/futurerenderer.dart';

class TrackViewsSwitch extends ChangeNotifier {
  int currentIndex = 0;
  final List<TrackData> exposed;
  TrackViewsSwitch({required this.exposed});

  static List<TrackData> wmp() {
    return [TrackData.wheel, TrackData.map, TrackData.profile];
  }

  void cycle() {
    currentIndex++;
    if (currentIndex >= exposed.length) {
      currentIndex = 0;
    }
    notifyListeners();
  }

  TrackData currentData() {
    return exposed[currentIndex];
  }

  void changeCurrent(TrackData d) {
    currentIndex = exposed.indexOf(d);
    notifyListeners();
  }
}
