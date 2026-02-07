import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/screens/load/load_screen.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

class DesktopShell extends StatelessWidget {
  const DesktopShell({super.key});

  @override
  Widget build(BuildContext context) {
    return DesktopScreen();
  }
}

class DesktopScreen extends StatelessWidget {
  const DesktopScreen({super.key});

  @override
  Widget build(BuildContext context) {
    FociModel foci = Provider.of<FociModel>(context);
    debugPrint("mobile focus on: ${foci.foci}");
    RootModel root = Provider.of<RootModel>(context);
    if (foci.contains(ScreenFocus.load)) {
      return LoadScreen(userInput: root.userInput!);
    }
    if (foci.contains(ScreenFocus.overview)) {
      return WheelScreen();
    }
    return HomeScreen();
  }
}
