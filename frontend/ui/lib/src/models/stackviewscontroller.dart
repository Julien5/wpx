import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:wpx/src/models/futurerenderer.dart';

class StackViewsController extends ChangeNotifier {
  int currentIndex = 0;
  final List<BridgeRenderFunction> exposed;
  final Map<BridgeRenderFunction, Size>? sizes;
  final Map<BridgeRenderFunction, double>? scales;

  StackViewsController({required this.exposed, this.sizes, this.scales});

  static List<BridgeRenderFunction> wmp() {
    return [
      BridgeRenderFunction.wheel,
      BridgeRenderFunction.map,
      BridgeRenderFunction.profile,
    ];
  }

  @override
  void notifyListeners() {
    debugPrint("ScreenConfiguration notifies");
    super.notifyListeners();
  }

  void cycle() {
    currentIndex++;
    if (currentIndex >= exposed.length) {
      currentIndex = 0;
    }
    developer.log("[1]currentIndex:$currentIndex");
    notifyListeners();
  }

  BridgeRenderFunction currentData() {
    return exposed[currentIndex];
  }

  void changeCurrent(BridgeRenderFunction d) {
    currentIndex = exposed.indexOf(d);
    developer.log("[2]currentIndex:$currentIndex");
    notifyListeners();
  }
}
