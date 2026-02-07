import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:ui/src/screens/shell/screen_shell.dart';

class Routes {
  static const String home = "/home";
  static const String load = "/load";
  static const String overview = "/overview";
  static const String usersteps = "/usersteps";
  static const String controls = "/controls";
  static const String settings = "/settings";
}

GoRouter getRouter() {
  debugPrint("CREATE ROUTER");
  return GoRouter(
    initialLocation: Routes.home,
    routes: [
      GoRoute(
        path: Routes.home,
        builder: (context, state) {
          debugPrint("home:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.load,
        builder: (context, state) {
          debugPrint("load:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.overview,
        builder: (context, state) {
          debugPrint("overview:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.usersteps,
        builder: (context, state) {
          debugPrint("usersteps:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.controls,
        builder: (context, state) {
          debugPrint("controls:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
      GoRoute(
        path: Routes.settings,
        builder: (context, state) {
          debugPrint("settings:${state.matchedLocation}");
          return ScreenShell();
        },
      ),
    ],
  );
}

String djoin(String a, String b) {
  return "$a/$b";
}

void goto(BuildContext ctx, String path) {
  developer.log("GOTO:$path");
  if (path.startsWith("/")) {
    ctx.go(path);
    return;
  }
  final currentLocation = GoRouterState.of(ctx).matchedLocation;
  final fullPath = '$currentLocation/$path';
  ctx.go(fullPath);
}

void pushto(BuildContext ctx, String path) {
  developer.log("PUSH:$path");
  if (path.startsWith("/")) {
    ctx.push(path);
    return;
  }
  final currentLocation = GoRouterState.of(ctx).matchedLocation;
  final fullPath = '$currentLocation/$path';
  ctx.push(fullPath);
}
