import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/screens/controls/controls_screen.dart';
import 'package:wpx/src/screens/home/home_screen.dart';
import 'package:wpx/src/screens/load/load_screen.dart';
import 'package:wpx/src/screens/settings/settings_screen.dart';
import 'package:wpx/src/screens/usersteps/usersteps_screen.dart';
import 'package:wpx/src/screens/wheel/wheel_screen.dart';

class MobileShell extends StatelessWidget {
  const MobileShell({super.key});

  @override
  Widget build(BuildContext context) {
    return MobileScreen();
  }
}

class MobileScreen extends StatelessWidget {
  const MobileScreen({super.key});

  @override
  Widget build(BuildContext context) {
    FociModel foci = context.watch<FociModel>();
    debugPrint("mobile focus on: ${foci.foci}");
    if (foci.contains(ScreenFocus.load)) {
      RootModel root = context.read<RootModel>();
      return LoadScreen(userInput: root.userInput!);
    }
    if (foci.contains(ScreenFocus.settings)) {
      return SettingsScreen();
    }
    if (foci.contains(ScreenFocus.controls)) {
      return ControlsScreen();
    }
    if (foci.contains(ScreenFocus.usersteps)) {
      return UserStepsScreen();
    }
    if (foci.contains(ScreenFocus.overview)) {
      return WheelScreen();
    }
    if (foci.contains(ScreenFocus.home)) {
      return HomeScreen();
    }
    developer.log("!!!! [NO SCREEN FOR ${foci.foci}] !!!!");
    assert(false);
    return HomeScreen();
  }
}
