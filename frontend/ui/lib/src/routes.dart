import 'package:flutter/material.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/screens/export/export_screen.dart';
import 'package:ui/src/screens/interactive/interactive_screen.dart';
import 'package:ui/src/screens/segments/segments_screen.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

class RouteManager {
  static const String home = '/';
  static const String wheelView = '/wheel';
  static const String settingsView = '/settings';
  static const String segmentsView = '/segments';
  static const String exportView = '/export';
  static const String interactiveView = '/interactive';

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
        return MaterialPageRoute(builder: (_) => WheelScreen());

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
