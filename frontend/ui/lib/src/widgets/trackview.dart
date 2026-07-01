import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/widgets/future_rendering_widget.dart';

class RendererParameters {
  final Kinds kinds;
  final BridgeRenderFunction trackData;
  const RendererParameters({required this.kinds, required this.trackData});
}

class TrackView extends StatelessWidget {
  final BridgeRenderFunction trackData;
  final Size? svgSize;
  const TrackView({super.key, required this.trackData, this.svgSize});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        FutureRenderer renderer = context.watch();
        Size size =
            svgSize != null
                ? svgSize!
                : Size(constraints.maxWidth, constraints.maxHeight);
        renderer.setSize(trackData, size);
        return FutureRenderingWidget(trackData: trackData, interactive: false);
      },
    );
  }
}
