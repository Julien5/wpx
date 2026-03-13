import 'dart:developer' as developer;

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/stackviewscontroller.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';

class SimpleTrackView extends StatefulWidget {
  final RendererParameters rendererParameters;
  const SimpleTrackView({super.key, required this.rendererParameters});

  static SimpleTrackView make(Set<Kind> kinds, TrackData trackData) {
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

class MaybeModelInserter extends StatelessWidget {
  final Widget child;
  final ChangeNotifier? maybe;

  const MaybeModelInserter({super.key, required this.child, this.maybe});

  @override
  Widget build(BuildContext context) {
    if (maybe == null) {
      return child;
    }
    // use the internal renderer
    return ChangeNotifierProvider.value(value: maybe!, child: child);
  }
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

  Widget buildd(BuildContext context) {
    Provider.of<SegmentModel>(context);
    TrackData data = widget.rendererParameters.trackData;
    StackViewsController viewSwitch = context.watch<StackViewsController>();

    return MaybeModelInserter(
      maybe: internalRenderer,
      child: LayoutBuilder(
        builder: (BuildContext context, BoxConstraints constraints) {
          Size maxSize = Size(constraints.maxWidth, constraints.maxHeight);
          Size? size;
          if (viewSwitch.sizes != null) {
            size = viewSwitch.sizes![data];
          }
          if (viewSwitch.scales != null && viewSwitch.scales![data] != null) {
            size = maxSize * viewSwitch.scales![data]!;
          }
          return TrackView(
            trackData: widget.rendererParameters.trackData,
            svgSize: size,
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    TrackData data = widget.rendererParameters.trackData;
    StackViewsController viewSwitch = context.watch<StackViewsController>();
    Size? size = viewSwitch.sizes != null ? viewSwitch.sizes![data] : null;
    TrackView trackView = TrackView(
      trackData: widget.rendererParameters.trackData,
      svgSize: size,
    );
    if (internalRenderer == null) {
      return trackView;
    }
    // use the internal renderer
    return ChangeNotifierProvider.value(
      value: internalRenderer!,
      builder: (context, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            debugPrint(
              "simpletrackview build ${internalRenderer!.clients} constraint:$constraints",
            );
            Size maxSize = Size(constraints.maxWidth, constraints.maxHeight);
            if (size == null &&
                viewSwitch.scales != null &&
                viewSwitch.scales![data] != null) {
              size = maxSize * viewSwitch.scales![data]!;
            }

            return TrackView(
              trackData: widget.rendererParameters.trackData,
              svgSize: size,
            );
          },
        );
      },
    );
  }
}
