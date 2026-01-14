import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class RendererParameters {
  final Set<InputType> kinds;
  final TrackData trackData;
  const RendererParameters({required this.kinds, required this.trackData});
  ValueKey createKey() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return ValueKey('${trackData.toString()}|${sortedKinds.join(",")}');
  }
}

class TrackView extends StatefulWidget {
  final RendererParameters parameters;
  const TrackView({super.key, required this.parameters});

  static TrackView make(Set<InputType> kinds, TrackData trackData) {
    RendererParameters p = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return TrackView(key: p.createKey(), parameters: p);
  }

  @override
  State<TrackView> createState() => _TrackViewState();
}

class _TrackViewState extends State<TrackView> {
  VisibilityInfo? visibilityInfo;
  FutureRenderer? futureRenderer;

  // The argument type is BuildContext, but using it yields
  // a crash. Dont ask me why.
  FutureRenderer _createRenderer(_) {
    SegmentModel model = Provider.of<SegmentModel>(context);
    futureRenderer = model.makeRenderer(
      widget.parameters.kinds,
      widget.parameters.trackData,
    );
    return futureRenderer!;
  }

  FutureRenderer _onSegmentModelChanged(FutureRenderer? renderer) {
    renderer!.reset();
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
    if (futureRenderer == null) {
      return;
    }
    futureRenderer!.setSize(visibilityInfo!.size);
    startRendererIfNeeded();
  }

  // takes visibility and renderer dirtyness into account.
  void startRendererIfNeeded() {
    if (futureRenderer == null) {
      return;
    }
    bool needed =
        visibilityInfo != null &&
        visibilityInfo!.visibleFraction > 0 &&
        futureRenderer!.needsStart();
    if (needed) {
      futureRenderer!.start();
      assert(!futureRenderer!.needsStart());
    }
  }

  @override
  Widget build(BuildContext ctx) {
    // reacts on change in the segmentmodel..
    SegmentModel model = Provider.of<SegmentModel>(ctx);
    Widget innerWidget = LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        return VisibilityDetector(
          // widget.key! causes an initial rendering problem in PDF
          // UniqueKey() causes flicker when adjusting the speed in WheelScreen
          key: widget.key!,
          onVisibilityChanged: _onVisibilityChanged,
          child: FutureRenderingWidget(interactive: false),
        );
      },
    );
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: model),
        ChangeNotifierProxyProvider<SegmentModel, FutureRenderer>(
          create: _createRenderer,
          update: (context, segment, futureRenderer) {
            return _onSegmentModelChanged(futureRenderer);
          },
        ),
      ],
      child: innerWidget,
    );
  }
}
