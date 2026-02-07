import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class EventModel extends ChangeNotifier {
  final bridge.Bridge backend;
  late Stream<String> _stream;
  String event = "";

  EventModel({required this.backend}) {
    _stream = backend.setSink();
    _stream.listen((data) {
      developer.log("EventModel.listen:$data");
      onEvent(data);
    });
  }

  void onEvent(String data) {
    developer.log("onEvent:$data");
    event = data;
    notifyListeners();
  }

  String get() {
    return event;
  }
}

class UserInput {
  List<int>? bytes;
  String? filename;
  bool demo = false;

  static UserInput makeFromBytes(List<int> bytes) {
    var ret = UserInput();
    ret.bytes = bytes;
    return ret;
  }

  static UserInput makeDemo() {
    var ret = UserInput();
    ret.demo = true;
    return ret;
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
    notifyListeners();
  }

  void notify() {
    notifyListeners();
  }
}

enum ScreenFocus { home, load, overview, usersteps, controls, pdf }

class Focii {
  final Set<ScreenFocus> focii;

  Focii({required this.focii});

  static Focii fromRoute(String path) {
    List<String> parts = path.split('/').where((s) => s.isNotEmpty).toList();
    debugPrint("parts:$parts");
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
        default:
          developer.log("what is [$seg] ?");
          assert(false);
          break;
      }
    }
    if (result.isEmpty) {
      result.add(ScreenFocus.home);
    }

    return Focii(focii: result);
  }

  bool contains(ScreenFocus f) {
    return focii.contains(f);
  }
}
