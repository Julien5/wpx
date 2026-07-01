import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';

class Routes {
  static const String home = "/";
  static const String load = "/load";
  static const String overview = "/overview";
  static const String usersteps = "/usersteps";
  static const String controls = "/controls";
  static const String settings = "/settings";
}

GoRouter getRouter() {
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

void gotoOverview(BuildContext ctx) {
  FociModel fociModel = ctx.read<FociModel>();
  ScreenConfiguration config = ctx.read<ScreenConfiguration>();
  fociModel.setFocus(ScreenFocus.overview);
  if (config.isMobile()) {
    ctx.go(Routes.overview);
  }
}

void gotoUserSteps(BuildContext ctx) {
  debugPrint("gotoUserSteps");
  FociModel fociModel = ctx.read<FociModel>();
  ScreenConfiguration config = ctx.read<ScreenConfiguration>();
  fociModel.addFocus(ScreenFocus.usersteps);
  if (config.isMobile()) {
    ctx.go(Routes.usersteps);
  }
}

void gotoPDF(BuildContext ctx) {
  debugPrint("gotoPDF");
  FociModel fociModel = ctx.read<FociModel>();
  ScreenConfiguration config = ctx.read<ScreenConfiguration>();
  fociModel.addFocus(ScreenFocus.settings);
  if (config.isMobile()) {
    ctx.go(Routes.settings);
  }
}

void gotoControls(BuildContext ctx) {
  debugPrint("gotoControls");
  FociModel fociModel = ctx.read<FociModel>();
  ScreenConfiguration config = ctx.read<ScreenConfiguration>();
  fociModel.addFocus(ScreenFocus.controls);
  if (config.isMobile()) {
    ctx.go(Routes.controls);
  }
}

void gotoHome(BuildContext ctx) {
  debugPrint("goto home");
  FociModel fociModel = ctx.read<FociModel>();
  fociModel.setFocus(ScreenFocus.home);
  ctx.go(Routes.home);
}

void gotoLoad(BuildContext ctx) {
  debugPrint("goto load");
  FociModel fociModel = ctx.read<FociModel>();
  fociModel.setFocus(ScreenFocus.load);
  ctx.go(Routes.load);
}

// ignore: unused_element
void _pushto(BuildContext ctx, String path) {
  developer.log("PUSH:$path");
  if (path.startsWith("/")) {
    ctx.push(path);
    return;
  }
  final currentLocation = GoRouterState.of(ctx).matchedLocation;
  final fullPath = '$currentLocation/$path';
  ctx.push(fullPath);
}
