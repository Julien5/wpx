import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/segments/future_rendering_widget.dart';

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

class WheelScreen extends StatefulWidget {
  const WheelScreen({super.key});

  @override
  State<WheelScreen> createState() => _WheelScreenState();
}

class _WheelScreenState extends State<WheelScreen> {
  void gotoSettings(BuildContext ctx) {
    Navigator.of(ctx).pushNamed(RouteManager.settingsView);
  }

  Future<void> gotoUserSteps(BuildContext ctx) async {
    await Navigator.of(ctx).pushNamed(RouteManager.userStepsView);
    // rebuild
    setState(() {});
  }

  Future<void> gotoControls(BuildContext ctx) async {
    await Navigator.of(ctx).pushNamed(RouteManager.controlsView);
    // rebuild
    setState(() {});
  }

  @override
  Widget build(BuildContext ctx) {
    Widget pdfButton = ElevatedButton(
      onPressed: () => gotoSettings(ctx),
      child: const Text("Feuille de route"),
    );

    Widget controlsButtons = ElevatedButton(
      onPressed: () => gotoControls(ctx),
      child: const Text("Control Points"),
    );

    Widget userStepsButton = ElevatedButton(
      onPressed: () => gotoUserSteps(ctx),
      child: const Text("Pacing Points"),
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
              vspace,
              controlsButtons,
              vspace,
              userStepsButton,
              vspace,
              pdfButton,
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
