import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
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
      Padding(
        padding: const EdgeInsets.all(15),
        child: StatisticsWidget(
          onPacingPointPressed: () => gotoUserSteps(ctx),
          onControlsPointPressed: () => gotoControls(ctx),
          onPagesPressed: () => gotoSettings(ctx),
        ),
      ),
    ];
    Widget leftCol = ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: 400,
        maxWidth: 400,
        maxHeight: screen.height,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: leftChildren,
      ),
    );

    Widget mwrow = Row(
      children: [
        Expanded(child: TrackView.make({InputType.userStep}, TrackData.map)),
        Expanded(child: TrackView.make({InputType.userStep}, TrackData.wheel)),
      ],
    );

    List<Widget> rightChildren = [
      Expanded(child: TrackView.make({InputType.userStep}, TrackData.profile)),
      Expanded(child: mwrow),
      Expanded(child: SizedBox(height: 50)),
    ];

    Widget rightCol = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: 800),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: rightChildren,
      ),
    );

    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Row(children: [leftCol, div, rightCol]),
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
