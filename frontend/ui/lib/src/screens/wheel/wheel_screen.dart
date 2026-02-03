import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_screen.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/usersteps/usersteps_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/export.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';
import 'package:ui/src/widgets/small.dart';

class _WheelScaffold extends StatelessWidget {
  void gotoSettings(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => SettingsScreen(model: model)),
    );
  }

  void gotoUserSteps(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    TrackViewsSwitch viewsSwitch = Provider.of<TrackViewsSwitch>(
      ctx,
      listen: false,
    );
    Navigator.push(
      ctx,
      MaterialPageRoute(
        builder:
            (context) =>
                UserStepsScreen(model: model, multiTrackModel: viewsSwitch),
      ),
    );
  }

  void gotoControls(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    TrackViewsSwitch viewsSwitch = Provider.of<TrackViewsSwitch>(
      ctx,
      listen: false,
    );
    Navigator.push(
      ctx,
      MaterialPageRoute(
        builder:
            (context) =>
                ControlsScreen(model: model, multiTrackModel: viewsSwitch),
      ),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = SmallCentralWidget(
      child: StatisticsWidget(
        onPacingPointPressed: () => gotoUserSteps(ctx),
        onControlsPointPressed: () => gotoControls(ctx),
        onPagesPressed: () => gotoSettings(ctx),
      ),
    );

    //ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 400),
        child: TrackGraphicsRow(kinds: allkinds(), maxHeight: 300),
      ),
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 400),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          children: [
            statisticsCard,
            Center(child: ExportButton(text: "export zip", type: Type.zip)),
          ],
        ),
      ),
    ];

    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.start,
        children: children,
      ),
    );
  }
}

class _WheelScreenProviders extends MultiProvider {
  _WheelScreenProviders({required RootModel root, required Widget child})
    : super(
        providers: [
          ChangeNotifierProvider(
            create:
                (_) => SegmentModel(root: root, segment: root.trackSegment()),
          ),
          ChangeNotifierProvider(
            create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
          ),
        ],
        child: child,
      );
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return _WheelScreenProviders(root: root, child: _WheelScaffold());
  }
}
