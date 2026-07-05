import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
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
    final state = GoRouterState.of(context);
    final path = state.matchedLocation;
    final root = context.read<RootModel>();
    if (path == '/') {
      return HomeScreen();
    }
    if (path == '/load') {
      return LoadScreen(userInput: root.pendingContent()!);
    }
    // path == '/overview'
    switch (state.uri.queryParameters['mode']) {
      case 'usersteps':
        return UserStepsScreen();
      case 'controls':
        return ControlsScreen();
      case 'settings':
        return SettingsScreen();
      default:
        return WheelScreen();
    }
  }
}
