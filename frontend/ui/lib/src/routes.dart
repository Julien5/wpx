import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';

class Routes {
  static const String home = "/";
  static const String load = "/load";
  static const String overview = "/overview";
}

GoRouter getRouter() {
  return GoRouter(
    initialLocation: Routes.home,
    routes: [
      GoRoute(
        path: Routes.home,
        builder: (context, state) {
          debugPrint("home:${state.matchedLocation}");
          return ScreenShell(routerState: state);
        },
      ),
      GoRoute(
        path: Routes.load,
        builder: (context, state) {
          debugPrint("load:${state.matchedLocation}");
          return ScreenShell(routerState: state);
        },
      ),
      GoRoute(
        path: Routes.overview,
        builder: (context, state) {
          debugPrint("overview:${state.matchedLocation}");
          return ScreenShell(routerState: state);
        },
      ),
    ],
  );
}

void gotoOverview(BuildContext ctx) {
  ctx.go(Routes.overview);
}

void gotoUserSteps(BuildContext ctx) {
  debugPrint("gotoUserSteps");
  ctx.go('${Routes.overview}?mode=usersteps');
}

void gotoPDF(BuildContext ctx) {
  debugPrint("gotoPDF");
  ctx.go('${Routes.overview}?mode=settings');
}

void gotoControls(BuildContext ctx) {
  debugPrint("gotoControls");
  ctx.go('${Routes.overview}?mode=controls');
}

void gotoHome(BuildContext ctx) {
  debugPrint("goto home");
  ctx.read<RootModel>().getBackend().unload();
  ctx.go(Routes.home);
}

void gotoLoad(BuildContext ctx) {
  debugPrint("goto load");
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
