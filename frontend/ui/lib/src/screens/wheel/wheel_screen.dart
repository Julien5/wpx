import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_screen.dart';
import 'package:ui/src/screens/segments/future_rendering_widget.dart';
import 'package:ui/src/screens/usersteps/usersteps_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';

class WheelWidget extends StatefulWidget {
  final Set<InputType> kinds;
  const WheelWidget({super.key, required this.kinds});
  @override
  State<WheelWidget> createState() => _WheelWidgetState();
}

class _WheelWidgetState extends State<WheelWidget> {
  @override
  Widget build(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(context);
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        WheelRenderer wheelRenderer = model.createWheelRenderer(widget.kinds);
        wheelRenderer.setSize(Size(250, 250));
        return FutureRenderingWidget(future: wheelRenderer, interactive: false);
      },
    );
  }
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  void gotoSettings(BuildContext ctx) {
    Navigator.of(ctx).pushNamed(RouteManager.settingsView);
  }

  void gotoUserSteps(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => UserStepsProvider(model: model)),
    );
  }

  void gotoControls(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => ControlsProvider(model: model)),
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
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              WheelWidget(kinds: allkinds()),
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
      ),
    );
  }
}

class WheelProvider extends StatelessWidget {
  const WheelProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return ChangeNotifierProvider(
      create: (ctx) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return WheelScreen();
      },
    );
  }
}
