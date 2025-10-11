import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';

import 'future_rendering_widget.dart';

class MapConsumer extends StatelessWidget {
  const MapConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<MapRenderer>(
      builder: (context, mapRenderer, child) {
        return FutureRenderingWidget(future: mapRenderer);
      },
    );
  }
}

class MapScrollWidget extends StatelessWidget {
  const MapScrollWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: MapConsumer(),
    );
  }
}