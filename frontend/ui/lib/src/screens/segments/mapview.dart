import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';

class MapConsumer extends StatelessWidget {
  const MapConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<MapRenderer>(
      builder: (context, mapRenderer, child) {
        mapRenderer.setSize(Size(400, 400));
        return FutureRenderingWidget(interactive: false);
      },
    );
  }
}
