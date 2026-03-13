import 'dart:developer' as developer;

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
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

class _SimpleTrackViewState extends State<SimpleTrackView> {
  FutureRenderer? futureRenderer;

  @override
  void initState() {
    super.initState();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<ParameterModel>();
    SegmentModel segment = Provider.of<SegmentModel>(context, listen: false);
    futureRenderer ??= FutureRenderer(
      bridge: segment.backend,
      segment: segment.segment,
      kinds: widget.rendererParameters.kinds,
      clients: [widget.rendererParameters.trackData],
    );

    TrackViewsSwitch viewSwitch = context.watch<TrackViewsSwitch>();

    bool visible = (viewSwitch.currentData() == futureRenderer!.clients.first);
    futureRenderer!.setVisible(visible);
    futureRenderer!.updateSegment(segment.segment);
    futureRenderer!.reset();
    futureRenderer!.start();
  }

  @override
  void dispose() {
    super.dispose();
    futureRenderer!.dispose();
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    TrackData data = widget.rendererParameters.trackData;
    TrackViewsSwitch viewSwitch = context.watch<TrackViewsSwitch>();
    Size? size = viewSwitch.sizes != null ? viewSwitch.sizes![data] : null;
    return ChangeNotifierProvider.value(
      value: futureRenderer!,
      builder: (context, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            debugPrint(
              "simpletrackview build ${futureRenderer!.clients} constraint:$constraints",
            );
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
