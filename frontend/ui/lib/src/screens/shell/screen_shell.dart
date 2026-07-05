import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/screens/shell/desktop.dart';
import 'package:wpx/src/screens/shell/mobile.dart';

class ScreenShell extends StatelessWidget {
  final GoRouterState routerState;
  const ScreenShell({super.key, required this.routerState});

  @override
  Widget build(BuildContext context) {
    final path = routerState.matchedLocation;
    ScreenConfiguration screen = context.read<ScreenConfiguration>();
    return LayoutBuilder(
      builder: (context, constraints) {
        debugPrint("CONST: $constraints, path: $path");
        screen.updateConstraints(constraints);
        if (screen.isMobile()) {
          return MobileShell(state: routerState);
        }
        return DesktopShell(state: routerState);
      },
    );
  }
}
