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
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(context);
    return LayoutBuilder(
      builder: (context, constraints) {
        Future.microtask(() {
          if (!context.mounted) return;
          context.read<ScreenConfiguration>().updateConstraints(constraints);
        });

        final currentLocation = GoRouterState.of(context).matchedLocation;
        FociModel model = Provider.of<FociModel>(context, listen: false);
        model.load(currentLocation);
        if (screen.isMobile()) {
          return MobileShell();
        }
        return DesktopShell();
      },
    );
  }
}
