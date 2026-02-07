import 'package:flutter/material.dart';

enum DisplayMode { vertical, horizontal, desktop }

class ScreenConfiguration extends ChangeNotifier {
  DisplayMode _mode = DisplayMode.vertical;
  double _width = 0;
  double _height = 0;

  DisplayMode get mode => _mode;
  bool get isHorizontal => _mode == DisplayMode.horizontal;
  double get width => _width;
  double get height => _height;

  bool isMobile() {
    return mode != DisplayMode.desktop;
  }

  DisplayMode computeMode(double width, double height) {
    if (width > 900 && height > 700) {
      return DisplayMode.desktop;
    }
    if (width > height && width > 900) {
      return DisplayMode.horizontal;
    }
    return DisplayMode.vertical;
  }

  void updateConstraints(BoxConstraints constraints) {
    _height = constraints.maxHeight;
    _width = constraints.maxWidth;
    final newMode = computeMode(constraints.maxWidth, constraints.maxHeight);
    if (_mode != newMode) {
      _mode = newMode;
      notifyListeners();
    }
  }
}
