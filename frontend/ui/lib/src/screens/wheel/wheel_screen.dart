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
    pushto(ctx, Routes.settings);
  }

  void gotoUserSteps(BuildContext ctx) {
    pushto(ctx, Routes.usersteps);
  }

  void gotoControls(BuildContext ctx) {
    pushto(ctx, Routes.controls);
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = StatisticsWidget(
      onPacingPointPressed: () => gotoUserSteps(ctx),
      onControlsPointPressed: () => gotoControls(ctx),
      onPagesPressed: () => gotoSettings(ctx),
    );
    List<Widget> children = [
      statisticsCard,
      Center(child: ExportButton(text: "export zip", type: Type.zip)),
    ];

    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
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
