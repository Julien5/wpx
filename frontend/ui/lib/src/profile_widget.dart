import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';

class TrackWidget extends StatefulWidget {
  final TrackData trackData;
  const TrackWidget({super.key, required this.trackData});
  @override
  State<TrackWidget> createState() => TrackWidgetState();
}

class TrackWidgetState extends State<TrackWidget> {
  double currentEpsilon = 0;
  @override
  void initState() {
    super.initState();
    developer.log("TrackWidgetState init state");
    ensureStart();
  }

  void ensureStart() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      start();
    });
  }

  void start() {
    if (!mounted) {
      return;
    }
    final renderings = Segment.of(context);
    if (widget.trackData == TrackData.track) {
      if (renderings.track.needsStart()) {
        renderings.track.start();
      }
    }
    if (widget.trackData == TrackData.waypoints) {
      if (renderings.waypoints.needsStart()) {
        renderings.waypoints.start();
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final renderings = Segment.of(context);
    FutureRendering? sender;
    if (widget.trackData == TrackData.track) {
      sender = renderings.track;
    } else {
      sender = renderings.waypoints;
    }
    return ListenableBuilder(
      listenable: sender,
      builder: (context, _) {
        return buildWorker(context, sender!);
      },
    );
  }

  Widget? child;

  Widget buildWorker(BuildContext context, FutureRendering future) {
    if (child == null) {
      developer.log("START ${widget.trackData} ${future.id()}");
      child = Text("START ${widget.trackData} ${future.segment.id()}");
    }

    if (future.done()) {
      developer.log("SVG DONE ${widget.trackData} ${future.id()}");
      child = SvgPicture.string(future.result(), width: 600, height: 150);
    }
    developer.log("TrackWidgetState buildWorker");
    ensureStart();
    return SizedBox(width: 600.0, child: Column(children: [child!]));
  }
}

class SegmentStack extends StatefulWidget {
  const SegmentStack({super.key});

  @override
  State<SegmentStack> createState() => _SegmentStackState();
}

class _SegmentStackState extends State<SegmentStack> {
  @override
  Widget build(BuildContext context) {
    final renderings = Segment.of(context);
    return Stack(
      children: <Widget>[
        TrackWidget(trackData: TrackData.track),
        TrackWidget(trackData: TrackData.waypoints),
        Text("ID:${renderings.id()}"),
      ],
    );
  }
}
