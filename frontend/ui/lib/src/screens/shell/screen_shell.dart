import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/screens/shell/desktop.dart';
import 'package:wpx/src/screens/shell/mobile.dart';

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
    ScreenConfiguration screen = context.read<ScreenConfiguration>();
    return LayoutBuilder(
      builder: (context, constraints) {
        debugPrint(
          "CONST: $constraints, isTop: $isTopRoute, path: $currentLocation",
        );

        if (isTopRoute) {
          screen.updateConstraints(constraints);
        }
        if (screen.isMobile()) {
          return MobileShell();
        }
        return DesktopShell();
      },
    );
  }
}
