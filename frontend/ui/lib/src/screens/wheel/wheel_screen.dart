import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/export.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';

class _WheelScaffold extends StatelessWidget {
  void onPDFPressed(BuildContext ctx) {
    gotoPDF(ctx);
  }

  void onPacingPointPressed(BuildContext ctx) {
    gotoUserSteps(ctx);
  }

  void onControlsPointPressed(BuildContext ctx) {
    gotoControls(ctx);
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = OverviewWidget(
      onPacingPointPressed: () => onPacingPointPressed(ctx),
      onControlsPointPressed: () => onControlsPointPressed(ctx),
      onPDFPressed: () => onPDFPressed(ctx),
    );
    List<Widget> children = [
      statisticsCard,
      Center(child: ExportButton(text: "export zip")),
    ];

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: const Text('Overview'),
      ),
      body: AdaptiveLayout(
        topRow: TrackGraphicsRow(kinds: allkinds()),
        midChildren: children,
      ),
    );
  }
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  @override
  Widget build(BuildContext context) {
    context.watch<SegmentModel>();
    return _WheelScaffold();
  }
}
