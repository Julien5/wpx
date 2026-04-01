import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/screens/desktop/main_screen.dart';
import 'package:wpx/src/screens/home/home_screen.dart';
import 'package:wpx/src/screens/load/load_screen.dart';

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
    debugPrint("desktop focus on: ${foci.foci}");
    RootModel root = Provider.of<RootModel>(context);
    if (foci.contains(ScreenFocus.home)) {
      return HomeScreen();
    }
    if (foci.contains(ScreenFocus.load)) {
      return LoadScreen(userInput: root.userInput!);
    }
    return MainScreen();
  }
}
