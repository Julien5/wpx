import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' show Bridge;
import 'package:ui/src/screens/segments/future_rendering_widget.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class WheelWidget extends StatefulWidget {
  const WheelWidget({super.key});
  @override
  State<WheelWidget> createState() => _WheelWidgetState();
}

class _WheelWidgetState extends State<WheelWidget> {
  @override
  Widget build(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(context);
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        WheelRenderer wheelRenderer = model.createWheelRenderer();
        wheelRenderer.setSize(Size(250,250));
        return FutureRenderingWidget(future: wheelRenderer, interactive: false);
      },
    );
  }
}


class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Card infoCard = Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.all(1), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: Text("infocard"),
      ),
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Wheel')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              WheelWidget(),
              SizedBox(height: 150),
              UserStepsSliderProvider(),
              infoCard,
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
      create: (ctx) => SegmentModel(bridge,root.trackSegment()),
      builder: (context, child) {
        return WheelScreen();
      },
    );
  }
}