import 'dart:developer' as developer;

import 'package:flutter/material.dart';

enum DisplayMode { vertical, horizontal, mid, large }

class ScreenConfiguration extends ChangeNotifier {
  DisplayMode _mode = DisplayMode.vertical;
  double _width = 0;
  double _height = 0;

  DisplayMode get mode => _mode;
  bool get isHorizontal => _mode == DisplayMode.horizontal;
  double get width => _width;
  double get height => _height;

  DisplayMode computeMode(double width, double height) {
    if (width > 700 && height > 700) {
      return DisplayMode.large;
    }
    if (width > 500 && height > 500) {
      return DisplayMode.mid;
    }
    if (width > 500) {
      return DisplayMode.horizontal;
    }
    return DisplayMode.vertical;
  }

  void updateConstraints(BoxConstraints constraints) {
    _height = constraints.maxHeight;
    _width = constraints.maxWidth;
    final newMode = computeMode(constraints.maxWidth, constraints.maxHeight);
    developer.log("******* $_width x $_height");
    if (_mode != newMode) {
      _mode = newMode;
      developer.log("******* $_mode");
      notifyListeners();
    }
  }
}
