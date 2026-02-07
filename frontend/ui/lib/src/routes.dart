import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/shell/screen_shell.dart';

class Routes {
  static const String home = "/home";
  static const String load = "/load";
  static const String overview = "/overview";
  static const String usersteps = "usersteps";
  static const String controls = "controls";
  static const String settings = "settings";
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
          debugPrint("overview:${state.matchedLocation}");
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),
      GoRoute(
        path: join(Routes.overview, Routes.usersteps),
        builder: (context, state) {
          debugPrint("usersteps:${state.matchedLocation}");
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),
      GoRoute(
        path: join(Routes.overview, Routes.controls),
        builder: (context, state) {
          debugPrint("controls:${state.matchedLocation}");
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),
      GoRoute(
        path: join(Routes.overview, Routes.settings),
        builder: (context, state) {
          debugPrint("settings:${state.matchedLocation}");
          FociModel model = Provider.of<FociModel>(context, listen: false);
          model.load(state.matchedLocation);
          return ScreenShell();
        },
      ),
    ],
  );
}

String join(String a, String b) {
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
  ctx.go(fullPath); // '/overview/usersteps'
}

void pushto(BuildContext ctx, String path) {
  developer.log("GOTO:$path");
  if (path.startsWith("/")) {
    ctx.push(path);
    return;
  }
  final currentLocation = GoRouterState.of(ctx).matchedLocation;
  final fullPath = '$currentLocation/$path';
  ctx.push(fullPath); // '/overview/usersteps'
}
