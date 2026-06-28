import 'dart:developer' as developer;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

class UserInput {
  List<List<int>>? bytes;

  static UserInput makeFromBytes(List<List<int>> bytes) {
    var ret = UserInput();
    ret.bytes = bytes;
    return ret;
  }

  List<Uint8List> contents() {
    return bytes!.map((chunk) => Uint8List.fromList(chunk)).toList();
  }
}

class RootModel extends ChangeNotifier {
  final bridge.Bridge backend;
  UserInput? userInput;

  RootModel({required this.backend});

  bridge.Bridge getBackend() {
    return backend;
  }

  void setUserInput(UserInput u) {
    userInput = u;
    backend.unload();
    notifyListeners();
  }

  bool isLoaded() {
    return backend.isLoaded();
  }

  void notify() {
    notifyListeners();
  }
}

enum ScreenFocus { home, load, overview, usersteps, controls, settings }

class FociModel extends ChangeNotifier {
  Set<ScreenFocus> foci;

  FociModel({Set<ScreenFocus>? initialFoci})
    : foci = initialFoci ?? {ScreenFocus.home};

  void setFocus(ScreenFocus f) {
    if (foci.length == 1 && foci.contains(f)) {
      return;
    }
    foci.clear();
    foci.add(f);
    notifyListeners();
  }

  void addFocus(ScreenFocus f) {
    if (foci.contains(f)) {
      return;
    }
    if (f == ScreenFocus.usersteps) {
      foci.remove(ScreenFocus.settings);
      foci.remove(ScreenFocus.controls);
    }
    if (f == ScreenFocus.settings) {
      foci.remove(ScreenFocus.usersteps);
      foci.remove(ScreenFocus.controls);
    }
    if (f == ScreenFocus.settings) {
      foci.remove(ScreenFocus.usersteps);
      foci.remove(ScreenFocus.controls);
    }
    if (f == ScreenFocus.controls) {
      foci.remove(ScreenFocus.usersteps);
      foci.remove(ScreenFocus.settings);
    }
    foci.add(f);
    notifyListeners();
  }

  void removeFocus(ScreenFocus f) {
    if (!foci.contains(f)) {
      return;
    }
    foci.remove(f);
    assert(foci.isNotEmpty);
    notifyListeners();
  }

  bool hasOnly(ScreenFocus f) {
    if (foci.length != 1) {
      return false;
    }
    return foci.contains(f);
  }

  // ignore: unused_element
  void _loadPath(String path) {
    List<String> parts = path.split('/').where((s) => s.isNotEmpty).toList();
    debugPrint("parts:$parts");

    // preserve the / at the first element
    // because Routes.home = "/home";
    if (parts.isNotEmpty) {
      parts[0] = "/${parts[0]}";
    }

    final Set<ScreenFocus> result = {};

    for (final seg in parts) {
      switch (seg) {
        case Routes.home:
          result.add(ScreenFocus.home);
          break;
        case Routes.load:
          result.add(ScreenFocus.load);
          break;
        case Routes.overview:
          result.add(ScreenFocus.overview);
          break;
        case Routes.usersteps:
          result.add(ScreenFocus.usersteps);
          break;
        case Routes.controls:
          result.add(ScreenFocus.controls);
          break;
        case Routes.settings:
          result.add(ScreenFocus.settings);
          break;
        default:
          developer.log("[!!!] what is [$seg] ?? [!!!]");
          assert(false);
          break;
      }
    }
    if (result.isEmpty) {
      result.add(ScreenFocus.home);
    }
    foci = result;
  }

  bool contains(ScreenFocus f) {
    debugPrint("foci:$foci");
    return foci.contains(f);
  }
}
