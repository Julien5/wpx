//import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/segments/future_rendering_widget.dart';

class InteractiveMapView extends StatelessWidget {
  const InteractiveMapView({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<MapRenderer>(
      builder: (context, mapRenderer, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            mapRenderer.setSize(constraints.biggest);
            return FutureRenderingWidget(future: mapRenderer);
          },
        );
      },
    );
  }
}

class InteractiveConsumer extends StatelessWidget {
  const InteractiveConsumer({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(children: [Expanded(child: InteractiveMapView())]),
      ),
    );
  }
}

class InteractiveScreen extends StatelessWidget {
  const InteractiveScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    RootModel root = Provider.of<RootModel>(ctx);
    Segment trackSegment = root.trackSegment();
    return Scaffold(
      appBar: AppBar(title: const Text('Map')),
      body: ChangeNotifierProvider<MapRenderer>(
        create: (_) => MapRenderer(root.getBridge(), trackSegment),
        child: InteractiveConsumer(),
      ),
    );
  }
}
