import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/screens/shell/desktop.dart';
import 'package:ui/src/screens/shell/mobile.dart';

class ScreenShell extends StatelessWidget {
  const ScreenShell({super.key});

  @override
  Widget build(BuildContext context) {
    final currentLocation = GoRouterState.of(context).matchedLocation;
    final router = GoRouter.of(context);
    final isTopRoute =
        router
            .routerDelegate
            .currentConfiguration
            .matches
            .last
            .matchedLocation ==
        currentLocation;
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(context);
    FociModel model = Provider.of<FociModel>(context);
    return LayoutBuilder(
      builder: (context, constraints) {
        debugPrint(
          "CONST: $constraints, isTop: $isTopRoute, path: $currentLocation",
        );

        // Only update if this is the visible (top) route
        if (isTopRoute) {
          screen.updateConstraints(constraints);
          model.load(currentLocation);
        }
        if (screen.isMobile()) {
          return MobileShell();
        }
        return DesktopShell();
      },
    );
  }
}
