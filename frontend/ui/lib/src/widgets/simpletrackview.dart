import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:visibility_detector/visibility_detector.dart';

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
  VisibilityInfo? visibilityInfo;
  late final Key _visibilityKey;
  Timer? _debounce;

  @override
  void initState() {
    super.initState();
    _visibilityKey = UniqueKey();
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
      trackData: [widget.rendererParameters.trackData],
    );

    _onSegmentModelChanged(segment, futureRenderer!);
  }

  FutureRenderer _onSegmentModelChanged(
    SegmentModel segment,
    FutureRenderer renderer,
  ) {
    renderer.updateSegment(segment.segment);
    renderer.reset();
    startRendererIfNeeded();
    return renderer;
  }

  void _onVisibilityChanged(VisibilityInfo info) {
    visibilityInfo = null;
    if (!mounted) {
      return;
    }
    visibilityInfo = info;

    if (visibilityInfo!.visibleFraction == 0) {
      return;
    }

    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 250), () {
      if (visibilityInfo == null) {
        return;
      }
      if (!mounted) {
        return;
      }
      startRendererIfNeeded();
    });
  }

  void startRendererIfNeeded() {
    bool needed =
        visibilityInfo != null &&
        visibilityInfo!.visibleFraction > 0 &&
        futureRenderer!.needsStart();
    if (needed) {
      futureRenderer!.start();
    }
  }

  @override
  void dispose() {
    _debounce?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    return ChangeNotifierProvider.value(
      value: futureRenderer!,
      builder: (context, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            return VisibilityDetector(
              key: _visibilityKey,
              onVisibilityChanged: _onVisibilityChanged,
              child: TrackView(rendererParameters: widget.rendererParameters),
            );
          },
        );
      },
    );
  }
}
