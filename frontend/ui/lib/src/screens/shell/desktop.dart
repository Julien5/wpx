import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/screens/desktop/main_screen.dart';
import 'package:wpx/src/screens/home/home_screen.dart';
import 'package:wpx/src/screens/load/load_screen.dart';

class DesktopShell extends StatelessWidget {
  final GoRouterState state;
  const DesktopShell({super.key, required this.state});

  @override
  Widget build(BuildContext context) {
    final path = state.matchedLocation;
    final root = context.watch<RootModel>();
    if (path == '/') {
      return HomeScreen();
    }
    if (path == '/load') {
      return LoadScreen(userInput: root.pendingContent()!);
    }
    return MainScreen();
  }
}
