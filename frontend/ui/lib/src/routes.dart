import 'package:flutter/material.dart';
import 'package:ui/src/choose_data.dart';
import 'package:ui/src/export_widget.dart';
import 'package:ui/src/segments_widget.dart';
import 'package:ui/src/settings_widget.dart';

class RouteManager {
  static const String home = '/';
  static const String settingsView = '/settings';
  static const String segmentsView = '/segments';
  static const String exportView = '/export';

  static Route<dynamic> generateRoute(RouteSettings settings) {
    switch (settings.name) {
      case home:
        return MaterialPageRoute(builder: (_) => const HomePage());

      case segmentsView:
        return MaterialPageRoute(builder: (_) => SegmentsProviderWidget());

      case settingsView:
        return MaterialPageRoute(builder: (_) => SettingsWidget());

      case exportView:
        return MaterialPageRoute(builder: (_) => ExportProviderWidget());

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
