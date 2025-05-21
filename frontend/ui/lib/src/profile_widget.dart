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
    final renderings = RenderingsModel.of(context);
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
  void didUpdateWidget(covariant TrackWidget oldWidget) {
    final renderings = RenderingsModel.of(context);
    developer.log("DID UPDATE ${renderings.track.segment.id()}");
    start();
    super.didUpdateWidget(oldWidget);
  }

  @override // needed ?
  void didChangeDependencies() {
    final renderings = RenderingsModel.of(context);
    developer.log("DID CHANGE DEPS ${renderings.track.segment.id()}");
    start();
    super.didChangeDependencies();
  }

  @override
  Widget build(BuildContext context) {
    final renderings = RenderingsModel.of(context);
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
      child = Text("starting ${future.currentEpsilon()}...");
    }

    if (future.done()) {
      developer.log("SVG ${widget.trackData} ${future.segment.id()} ${future.result().length}");
      child = SvgPicture.string(future.result(), width: 600, height: 150);
    }
    //ensureStart();
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
    final renderings = RenderingsModel.of(context);
    return Stack(
      children: <Widget>[
        TrackWidget(trackData: TrackData.track),
        TrackWidget(trackData: TrackData.waypoints),
        Text("ID:${renderings.track.segment.id()}"),
      ],
    );
  }
}
