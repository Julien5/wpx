import 'package:flutter/material.dart';
import 'package:ui/src/screens/controls/controls_screen.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/screens/export/export_screen.dart';
import 'package:ui/src/screens/interactive/interactive_screen.dart';
import 'package:ui/src/screens/segments/segments_screen.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/usersteps/usersteps_table.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

import 'screens/usersteps/usersteps_screen.dart';

class RouteManager {
  static const String home = '/';
  static const String wheelView = '/wheel';
  static const String settingsView = '/settings';
  static const String segmentsView = '/segments';
  static const String exportView = '/export';
  static const String userStepsView = '/usersteps';
  static const String userStepsTable = '/usersteps/table';
  static const String interactiveView = '/interactive';
  static const String controlsView = '/controls';

  static Route<dynamic> generateRoute(RouteSettings settings) {
    switch (settings.name) {
      case home:
        return MaterialPageRoute(builder: (_) => const HomeScreen());

      case segmentsView:
        return MaterialPageRoute(builder: (_) => SegmentsScreen());

      case settingsView:
        return MaterialPageRoute(builder: (_) => SettingsScreen());

      case exportView:
        return MaterialPageRoute(builder: (_) => ExportScreen());

      case interactiveView:
        return MaterialPageRoute(builder: (_) => InteractiveScreen());

      case wheelView:
        return MaterialPageRoute(builder: (_) => WheelProvider());

      case userStepsView:
        return MaterialPageRoute(builder: (_) => UserStepsProvider());

      case controlsView:
        return MaterialPageRoute(builder: (_) => ControlsProvider());

      case userStepsTable:
        return MaterialPageRoute(
          builder: (_) => UserStepsTableScreen(),
          settings: settings,
        );

      default:
        return MaterialPageRoute(
          builder:
              (_) => Scaffold(
                body: Center(
                  child: Text('No route defined for ${settings.name}'),
                ),
              ),
        );
    }
  }
}
