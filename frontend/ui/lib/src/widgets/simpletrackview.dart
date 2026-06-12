import 'dart:developer' as developer;

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/widgets/trackview.dart';

class SimpleTrackView extends StatefulWidget {
  final RendererParameters rendererParameters;
  const SimpleTrackView({super.key, required this.rendererParameters});

  static SimpleTrackView make(Kinds kinds, BridgeRenderFunction trackData) {
    developer.log("[SimpleTrackView make] $trackData");
    RendererParameters parameters = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return SimpleTrackView(rendererParameters: parameters);
  }

  @override
  State<SimpleTrackView> createState() => _SimpleTrackViewState();
}

class _SimpleTrackViewState extends State<SimpleTrackView> {
  @override
  void initState() {
    super.initState();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<ParameterModel>();
    FutureRenderer? externalRenderer = context.read<FutureRenderer?>();
    assert(externalRenderer != null);
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    BridgeRenderFunction data = widget.rendererParameters.trackData;
    StackViewsController controller = context.watch<StackViewsController>();
    // honor controller.sizes
    Size? size = controller.sizes != null ? controller.sizes![data] : null;
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size maxSize = Size(constraints.maxWidth, constraints.maxHeight);
        // honor controller.scales (if they are set)
        if (size == null &&
            controller.scales != null &&
            controller.scales![data] != null) {
          size = maxSize * controller.scales![data]!;
        }
        return TrackView(
          trackData: widget.rendererParameters.trackData,
          svgSize: size,
        );
      },
    );
  }
}
