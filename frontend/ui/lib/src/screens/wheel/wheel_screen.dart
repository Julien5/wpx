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
  void gotoSettings(BuildContext ctx) {
    goto(ctx, Routes.settings);
  }

  void gotoUserSteps(BuildContext ctx) {
    goto(ctx, Routes.usersteps);
  }

  void gotoControls(BuildContext ctx) {
    goto(ctx, Routes.controls);
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = OverviewWidget(
      onPacingPointPressed: () => gotoUserSteps(ctx),
      onControlsPointPressed: () => gotoControls(ctx),
      onPDFPressed: () => gotoSettings(ctx),
    );
    List<Widget> children = [
      statisticsCard,
      Center(child: ExportButton(text: "export zip", type: Type.zip)),
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
