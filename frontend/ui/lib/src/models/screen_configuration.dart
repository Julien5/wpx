import 'dart:developer' as developer;

import 'package:flutter/material.dart';

enum DisplayMode { vertical, horizontal, mid, large }

class ScreenConfiguration extends ChangeNotifier {
  DisplayMode _mode = DisplayMode.vertical;

  DisplayMode get mode => _mode;
  bool get isHorizontal => _mode == DisplayMode.horizontal;

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
    final newMode = computeMode(constraints.maxWidth, constraints.maxHeight);
    if (_mode != newMode) {
      developer.log("new mode: $newMode");
      _mode = newMode;
      // Only notifies listeners when the breakpoint is actually crossed
      notifyListeners();
    }
  }
}
