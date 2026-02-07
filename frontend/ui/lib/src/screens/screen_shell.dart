import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/screens/load/load_screen.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

class ScreenShell extends StatelessWidget {
  final Focii focii;

  const ScreenShell({super.key, required this.focii});

  @override
  Widget build(BuildContext context) {
    debugPrint("shell: ${focii.focii}");
    RootModel root = Provider.of<RootModel>(context);
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(context);
    return LayoutBuilder(
      builder: (context, constraints) {
        Future.microtask(() {
          if (!context.mounted) return;
          context.read<ScreenConfiguration>().updateConstraints(constraints);
        });
        if (focii.contains(ScreenFocus.overview)) {
          return WheelScreen();
        }
        if (focii.contains(ScreenFocus.load)) {
          return LoadScreen(userInput: root.userInput!);
        }
        if (screen.isMobile()) {
          return HomeScreen();
        }
        return HomeScreen();
      },
    );
  }
}
