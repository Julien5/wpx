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

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  void gotoSettings(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => SettingsProvider(model: model)),
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
                UserStepsProvider(model: model, multiTrackModel: viewsSwitch),
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
                ControlsProvider(model: model, multiTrackModel: viewsSwitch),
      ),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = Padding(
      padding: const EdgeInsets.all(1), // Add padding inside the card
      child: StatisticsWidget(
        onPacingPointPressed: () => gotoUserSteps(ctx),
        onControlsPointPressed: () => gotoControls(ctx),
        onPagesPressed: () => gotoSettings(ctx),
      ),
    );

    Widget vspace = SizedBox(height: 50);

    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            TrackGraphicsRow(kinds: allkinds(), height: 200),
            Expanded(child: vspace),
            Padding(padding: const EdgeInsets.all(16), child: statisticsCard),
            Expanded(child: vspace),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                Padding(
                  padding: const EdgeInsets.all(
                    16,
                  ), // Add padding inside the card
                  child: ExportButton(text: "export zip", type: Type.zip),
                ),
              ],
            ),
            Expanded(child: vspace),
          ],
        ),
      ),
    );
  }
}

class WheelScreenProviders extends MultiProvider {
  WheelScreenProviders({
    super.key,
    required RootModel root,
    required Widget child,
  }) : super(
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

class WheelProvider extends StatelessWidget {
  const WheelProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return WheelScreenProviders(root: root, child: WheelScreen());
  }
}
