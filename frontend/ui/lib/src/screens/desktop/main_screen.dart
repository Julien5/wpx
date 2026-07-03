import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/screens/desktop/central_panel.dart';
import 'package:wpx/src/screens/desktop/side_panel.dart';
import 'package:wpx/src/widgets/editable_text.dart';

class _MainScaffold extends StatelessWidget {
  Future<void> updateTrackFileName(BuildContext ctx, String newName) async {
    ctx.read<SegmentModel>().updateTrackfileName(newName: newName);
  }

  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = ctx.watch<ScreenConfiguration>();
    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );

    SegmentModel segmentModel = ctx.watch<SegmentModel>();
    String trackName = segmentModel.trackFileName();

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: WritableText(
          initialName: trackName,
          onSubmitted:
              (newName) async => await updateTrackFileName(ctx, newName),
        ),
      ),
      body: Row(
        children: [
          div,
          const SidePanel(width: 480),
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
