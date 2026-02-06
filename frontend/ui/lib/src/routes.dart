import 'package:flutter/material.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

class RouteManager {
  static const String home = '/';
  static const String wheelView = '/wheel';
  static const String settingsView = '/settings';

  static Route<dynamic> generateRoute(RouteSettings settings) {
    switch (settings.name) {
      case home:
        return MaterialPageRoute(builder: (_) => const HomeScreen());

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
