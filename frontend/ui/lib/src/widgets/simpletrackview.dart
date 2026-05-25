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

  static SimpleTrackView make(Kinds kinds, TrackData trackData) {
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
  FutureRenderer? internalRenderer;

  @override
  void initState() {
    super.initState();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<ParameterModel>();
    SegmentModel segment = Provider.of<SegmentModel>(context, listen: false);
    FutureRenderer? externalRenderer = context.read<FutureRenderer?>();
    if (externalRenderer == null) {
      internalRenderer ??= FutureRenderer(
        bridge: segment.backend,
        segment: segment.segment,
        kinds: widget.rendererParameters.kinds,
        clients: [widget.rendererParameters.trackData],
        name: "SimpleTrackView",
      );

      StackViewsController viewSwitch = context.read<StackViewsController>();

      assert(internalRenderer != null);
      bool visible =
          (viewSwitch.currentData() == internalRenderer!.clients.first);
      internalRenderer!.setVisible(visible);
      internalRenderer!.updateSegment(segment.segment);
      internalRenderer!.reset();
      internalRenderer!.start();
    }
  }

  @override
  void dispose() {
    super.dispose();
    if (internalRenderer != null) {
      internalRenderer!.dispose();
    }
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    TrackData data = widget.rendererParameters.trackData;
    StackViewsController controller = context.watch<StackViewsController>();
    // honor controller.sizes
    Size? size = controller.sizes != null ? controller.sizes![data] : null;
    Widget builder = LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size maxSize = Size(constraints.maxWidth, constraints.maxHeight);
        // honor controller.scales (if sizes where not set, sizes have predence)
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

    if (internalRenderer == null) {
      return builder;
    }

    // use the internal renderer
    return ChangeNotifierProvider.value(
      value: internalRenderer!,
      builder: (context, child) {
        return builder;
      },
    );
  }
}
