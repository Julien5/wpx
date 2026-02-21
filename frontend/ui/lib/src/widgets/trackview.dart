import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class RendererParameters {
  final Set<Kind> kinds;
  final TrackData trackData;
  const RendererParameters({required this.kinds, required this.trackData});
  ValueKey createKey() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return ValueKey('${trackData.toString()}|${sortedKinds.join(",")}');
  }
}

class TrackView extends StatefulWidget {
  const TrackView({super.key});

  @override
  State<TrackView> createState() => _TrackViewState();
}

class _TrackViewState extends State<TrackView> {
  VisibilityInfo? visibilityInfo;
  late final Key _visibilityKey;
  Timer? _debounce;

  @override
  void initState() {
    super.initState();
    _visibilityKey = UniqueKey();
  }

  FutureRenderer _onSegmentModelChanged(
    SegmentModel segment,
    FutureRenderer? renderer,
  ) {
    assert(renderer != null);
    renderer!.updateSegment(segment.segment);
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

    FutureRenderer futureRenderer = Provider.of<FutureRenderer>(
      context,
      listen: false,
    );
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 150), () {
      if (visibilityInfo == null) {
        return;
      }
      if (!mounted) {
        return;
      }
      Size size = visibilityInfo!.size;
      TrackData currentData = futureRenderer.trackData;
      ScreenConfiguration screen = Provider.of<ScreenConfiguration>(
        context,
        listen: false,
      );
      if (screen.isMobile()) {
        if (currentData == TrackData.map || currentData == TrackData.profile) {
          size = size * 1.5;
        }
      }
      futureRenderer.setSize(size);
      startRendererIfNeeded();
    });
  }

  // takes visibility and renderer dirtyness into account.
  void startRendererIfNeeded() {
    FutureRenderer futureRenderer = Provider.of<FutureRenderer>(
      context,
      listen: false,
    );

    bool needed =
        visibilityInfo != null &&
        visibilityInfo!.visibleFraction > 0 &&
        futureRenderer.needsStart();
    debugPrint("1:${visibilityInfo != null}");
    if (visibilityInfo != null) {
      debugPrint(
        "2:${visibilityInfo!.visibleFraction > 0} and 3:${futureRenderer.needsStart()}",
      );
    }
    debugPrint("3:${futureRenderer.needsStart()}");
    if (needed) {
      futureRenderer.start();
      // this assert fails, sometimes, when the screen is resized quickly
      // assert(!futureRenderer!.needsStart());
    }
  }

  @override
  Widget build(BuildContext ctx) {
    // reacts on change in the segmentmodel..
    SegmentModel segmentModel = Provider.of<SegmentModel>(ctx);
    Widget innerWidget = LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        return VisibilityDetector(
          // widget.key! causes an initial rendering problem in PDF
          // UniqueKey() causes flicker when adjusting the speed in WheelScreen
          // => we use a specific key
          key: _visibilityKey,
          onVisibilityChanged: _onVisibilityChanged,
          child: FutureRenderingWidget(interactive: false),
        );
      },
    );
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: segmentModel),
        ChangeNotifierProxyProvider2<
          SegmentModel,
          ParameterModel,
          FutureRenderer
        >(
          create: (context) => context.read<FutureRenderer>(),
          update: (context, segment, parameter, futureRenderer) {
            developer.log(
              "[update => segment:${segment.segment.id()} ${futureRenderer!.kinds} ${futureRenderer.trackData}]",
            );
            segment.debug();
            WidgetsBinding.instance.addPostFrameCallback((_) {
              _onSegmentModelChanged(segment, futureRenderer);
            });
            return futureRenderer;
          },
        ),
      ],
      child: innerWidget,
    );
  }
}
