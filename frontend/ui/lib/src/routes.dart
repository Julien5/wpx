import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/screen_shell.dart';

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
          return ScreenShell(focii: Focii.fromRoute(state.matchedLocation));
        },
      ),
      GoRoute(
        path: Routes.load,
        builder: (context, state) {
          debugPrint("load:${state.matchedLocation}");
          return ScreenShell(focii: Focii.fromRoute(state.matchedLocation));
        },
      ),

      GoRoute(
        path: Routes.overview,
        builder: (context, state) {
          return ScreenShell(focii: Focii.fromRoute(state.matchedLocation));
        },
        routes: [
          GoRoute(
            path: Routes.usersteps,
            builder: (context, state) {
              return ScreenShell(focii: Focii.fromRoute(state.matchedLocation));
            },
          ),
        ],
      ),
    ],
  );
}
