import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/future_rendering_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class SegmentStack extends StatefulWidget {
  const SegmentStack({super.key});

  @override
  State<SegmentStack> createState() => _SegmentStackState();
}

class _SegmentStackState extends State<SegmentStack> {
  RenderingsProvider? provider;
  double visibility = 0;

  @override
  void initState() {
    super.initState();
  }

  void postInit() {
    if (provider != null) {
      return;
    }
    provider ??= RenderingsProvider.of(context);
    provider!.renderings.track.addListener(() {
      onTrackChanged();
    });
    provider!.renderings.waypoints.addListener(() {
      onWaypointsChanged();
    });
  }

  @override
  void dispose() {
    provider!.renderings.track.removeListener(() {
      onTrackChanged();
    });
    provider!.renderings.waypoints.removeListener(() {
      onWaypointsChanged();
    });
    super.dispose();
  }

  // called immediately after initState() with safe context.
  @override
  void didChangeDependencies() {
    developer.log("[didChangeDependencies]");
    postInit();
    super.didChangeDependencies();
  }

  void onTrackChanged() {
    developer.log("[on track changed]");
    if (!mounted) {
      return;
    }
    update();
  }

  void onWaypointsChanged() {
    developer.log("[on WP changed]");
    if (!mounted) {
      return;
    }
    update();
  }

  void update() {
    developer.log("[update]");
    if (!mounted) {
      return;
    }
    if (visibility < 0.50) {
      return;
    }
    RenderingsProvider provider = RenderingsProvider.of(context);
    if (provider.renderings.track.needsStart()) {
      provider.renderings.track.start();
    }
    if (provider.renderings.waypoints.needsStart()) {
      provider.renderings.waypoints.start();
    }
    setState(() {});
  }

  void onVisibilityChanged(VisibilityInfo info) {
    developer.log("[on vis changed] ${info.visibleFraction}");
    if (!mounted) {
      return;
    }
    visibility = info.visibleFraction;
    update();
  }

  FutureRenderingWidget createFutureRenderingWidget(FutureRendering future) {
    return FutureRenderingWidget(future: future);
  }

  Widget createWaypointsWidget(FutureRendering future) {
    return VisibilityDetector(
      key: Key('id:${future.id()}'),
      onVisibilityChanged: onVisibilityChanged,
      child: FutureRenderingWidget(future: future),
    );
  }

  @override
  Widget build(BuildContext context) {
    final r = RenderingsProvider.of(context);
    Widget trackWidget = FutureRenderingWidget(future: r.renderings.track);
    Widget waypointsWidget = createWaypointsWidget(r.renderings.waypoints);
    return Stack(children: <Widget>[trackWidget, waypointsWidget]);
  }
}
