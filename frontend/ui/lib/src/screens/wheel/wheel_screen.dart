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
import 'package:ui/src/widgets/segmentgraphics.dart';

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  void gotoSettings(BuildContext ctx) {
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
                SettingsProvider(model: model, trackViewSwitch: viewsSwitch),
      ),
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
    Widget pdfButton = ElevatedButton(
      onPressed: () => gotoSettings(ctx),
      child: const Text("PDF"),
    );

    Widget controlsButtons = ElevatedButton(
      onPressed: () => gotoControls(ctx),
      child: const Text("Controls"),
    );

    Widget userStepsButton = ElevatedButton(
      onPressed: () => gotoUserSteps(ctx),
      child: const Text("Pacing"),
    );

    Card statisticsCard = Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.all(1), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: StatisticsWidget(),
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
            statisticsCard,
            Expanded(child: vspace),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [controlsButtons, userStepsButton, pdfButton],
            ),
            vspace,
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
             create: (_) => SegmentModel(root.getBridge(), root.trackSegment()),
           ),
           ChangeNotifierProvider(create: (_) => TrackViewsSwitch()),
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
