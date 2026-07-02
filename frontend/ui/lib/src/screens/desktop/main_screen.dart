import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/screens/desktop/central_panel.dart';
import 'package:wpx/src/screens/desktop/side_panel.dart';

class _MainScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = ctx.watch<ScreenConfiguration>();
    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: const Text('Overview'),
      ),
      body: Row(
        children: [
          div,
          SidePanel(width: 480),
          div,
          CentralPanel(width: screen.width - 500),
        ],
      ),
    );
  }
}

class MainScreen extends StatelessWidget {
  const MainScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return _MainScaffold();
  }
}
