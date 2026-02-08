import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/export.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';
import 'package:ui/src/widgets/trackview.dart';

class _LargeScaffold extends StatelessWidget {
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
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    List<Widget> leftChildren = [
      StatisticsWidget(
        onPacingPointPressed: () => gotoUserSteps(ctx),
        onControlsPointPressed: () => gotoControls(ctx),
        onPagesPressed: () => gotoSettings(ctx),
      ),
    ];
    Widget leftCol = ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: 400,
        maxWidth: 400,
        maxHeight: screen.height,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: leftChildren,
      ),
    );

    List<Widget> rightChildren = [
      Expanded(
        child: ConstrainedBox(
          constraints: BoxConstraints(maxHeight: 300),

          child: TrackView.make({InputType.userStep}, TrackData.profile),
        ),
      ),
      Expanded(
        child: ConstrainedBox(
          constraints: BoxConstraints(maxHeight: 500),
          child: TrackView.make({InputType.userStep}, TrackData.map),
        ),
      ),
    ];

    Widget rightCol = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: 600, maxHeight: screen.height),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: rightChildren,
      ),
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Row(children: [leftCol, rightCol]),
    );
  }
}

class _LargeScreenProviders extends MultiProvider {
  _LargeScreenProviders({required Widget child})
    : super(
        providers: [
          ChangeNotifierProvider(
            create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
          ),
        ],
        child: child,
      );
}

class LargeScreen extends StatelessWidget {
  const LargeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    context.watch<SegmentModel>();
    return _LargeScreenProviders(child: _LargeScaffold());
  }
}
