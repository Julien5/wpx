import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/future_rendering_widget.dart';
import 'package:ui/src/settings_widget.dart';
import 'package:ui/src/waypoints_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class SegmentStack extends StatelessWidget {
  const SegmentStack({super.key});

  @override
  Widget build(BuildContext context) {
    var stack = Align(
      alignment: Alignment.center,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Stack(children: <Widget>[TrackConsumer(), WaypointsConsumer()]),
      ),
    );
    var table = SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: WayPointsConsumer(),
    );
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [WidthSettings()],
        ),
        stack,
        Expanded(child: table),
      ],
    );
  }
}

class TrackConsumer extends StatelessWidget {
  const TrackConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<TrackRenderer>(
      builder: (context, trackRenderer, child) {
        return FutureRenderingWidget(future: trackRenderer);
      },
    );
  }
}

class WaypointsConsumer extends StatefulWidget {
  const WaypointsConsumer({super.key});

  @override
  State<WaypointsConsumer> createState() => _WaypointsConsumerState();
}

class _WaypointsConsumerState extends State<WaypointsConsumer> {
  double visibility = 0;

  void onVisibilityChanged(VisibilityInfo info) {
    if (!mounted) {
      return;
    }
    WaypointsRenderer wp = Provider.of<WaypointsRenderer>(
      context,
      listen: false,
    );
    developer.log(
      "[waypoint consumer] id:${wp.id()} vis:${info.visibleFraction}",
    );
    wp.updateVisibility(info.visibleFraction);
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<WaypointsRenderer>(
      builder: (context, waypointsRenderer, child) {
        // It would be more accurate to check visibility with a scroll controller
        // at the list view level. Because "Callbacks are not fired immediately
        // on visibility changes."
        return VisibilityDetector(
          key: Key('id:${waypointsRenderer.id()}'),
          onVisibilityChanged: onVisibilityChanged,
          child: FutureRenderingWidget(future: waypointsRenderer),
        );
      },
    );
  }
}
