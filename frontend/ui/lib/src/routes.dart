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

class RoutesHistory extends ChangeNotifier {
  List<String> history;

  RoutesHistory() : history = <String>[];

  void push(String route) {
    debugPrint("history push: $route");
    history.add(route);
  }

  String? last() {
    if (history.isEmpty) {
      return null;
    }
    return history.last;
  }
}

String? accessControl(RoutesHistory history, String wanted) {
  debugPrint("history=${history.history}");
  String? last = history.last();
  if (last == null) {
    if (wanted != Routes.home) {
      return Routes.home;
    } else {
      return null;
    }
  }
  // This does not cover all cases. For example home => overview
  // is accepted even when no SegmentModel exists. But this is ok
  // as a first flush.
  if (wanted == Routes.load && last != Routes.home) {
    return Routes.home;
  }
  return null;
}

GoRouter getRouter() {
  return GoRouter(
    initialLocation: Routes.home,
    redirect: (BuildContext context, GoRouterState state) {
      RoutesHistory history = context.read<RoutesHistory>();
      String wanted = state.matchedLocation;
      String? goto = accessControl(history, wanted);
      debugPrint("wanted:$wanted => goto:$goto");
      final effective = goto ?? wanted;
      if (history.last() != effective) {
        history.push(effective);
      }
      return goto;
    },
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

void gotoHome(BuildContext ctx) async {
  debugPrint("goto home");
  await ctx.read<RootModel>().getBackend().unload();
  if (!ctx.mounted) {
    return;
  }
  ctx.go(Routes.home);
}

void gotoLoad(BuildContext ctx) {
  debugPrint("goto load");
  ctx.go(Routes.load);
}
