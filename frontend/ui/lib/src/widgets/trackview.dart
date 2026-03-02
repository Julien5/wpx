import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';

class RendererParameters {
  final Set<Kind> kinds;
  final TrackData trackData;
  const RendererParameters({required this.kinds, required this.trackData});
}

class TrackView extends StatelessWidget {
  final RendererParameters rendererParameters;
  const TrackView({super.key, required this.rendererParameters});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        FutureRenderer renderer = Provider.of(context);
        Size size = Size(constraints.maxWidth, constraints.maxHeight);
        renderer.setSize(rendererParameters.trackData, size);
        return FutureRenderingWidget(
          trackData: rendererParameters.trackData,
          interactive: false,
        );
      },
    );
  }
}
