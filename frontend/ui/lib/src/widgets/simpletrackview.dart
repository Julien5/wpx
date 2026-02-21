import 'dart:developer' as developer;

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';

class SimpleTrackView extends StatelessWidget {
  final RendererParameters rendererParameters;
  const SimpleTrackView({super.key, required this.rendererParameters});

  FutureRenderer _create(BuildContext ctx, SegmentModel segment) {
    return segment.makeRenderer(
      rendererParameters.kinds,
      rendererParameters.trackData,
    );
  }

  static SimpleTrackView make(Set<Kind> kinds, TrackData trackData) {
    developer.log("[SimpleTrackView make] $trackData");
    RendererParameters parameters = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return SimpleTrackView(rendererParameters: parameters);
  }

  @override
  Widget build(BuildContext context) {
    SegmentModel segment = Provider.of<SegmentModel>(context);
    return ChangeNotifierProvider(
      create: (ctx) => _create(ctx, segment),
      builder: (context, child) {
        return TrackView(rendererParameters: rendererParameters);
      },
    );
  }
}
