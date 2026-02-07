import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/shell/screen_shell.dart';

class Routes {
  static const String home = '/';
  static const String load = '/load';
  static const String overview = '/overview';
  static const String usersteps = 'usersteps';
}

GoRouter getRouter() {
  debugPrint("CREATE ROUTER");
  return GoRouter(
    initialLocation: "/",
    routes: [
      GoRoute(
        path: Routes.home,
        builder: (context, state) {
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.load,
        builder: (context, state) {
          debugPrint("load:${state.matchedLocation}");
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),

      GoRoute(
        path: Routes.overview,
        builder: (context, state) {
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
        routes: [
          GoRoute(
            path: Routes.usersteps,
            builder: (context, state) {
              FociModel model = Provider.of<FociModel>(context, listen: false);
              model.load(state.matchedLocation);
              return ScreenShell();
            },
          ),
        ],
      ),
    ],
  );
}
