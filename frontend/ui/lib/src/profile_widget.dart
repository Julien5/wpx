import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';

class TrackWidget extends StatefulWidget {
  final FutureRendering future;
  const TrackWidget({super.key, required this.future});
  @override
  State<TrackWidget> createState() => TrackWidgetState();
}

class TrackWidgetState extends State<TrackWidget> {
  double currentEpsilon = 0;
  @override
  void initState() {
    super.initState();
    widget.future.start();
  }

  @override
  void didUpdateWidget(covariant TrackWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    developer.log("didUpdateWidget");
    if (!widget.future.started() && !widget.future.done()) {
      widget.future.start();
    }
  }

  /*@override
  void didChangeDependencies() {
    developer.log("didChangeDependencies");
    super.didChangeDependencies();
    if (!widget.future.started() && !widget.future.done()) {
      widget.future.start();
    }
  }*/

  @override
  Widget build(BuildContext context) {
    developer.log("buildd1 ${widget.future.currentEpsilon}...");
    return ListenableBuilder(
      listenable: widget.future,
      builder: (context, _) {
        return buildWorker(context);
      },
    );
  }

  Widget? child;

  Widget buildWorker(BuildContext context) {
    developer.log("SVG ..");
    if (child == null) {
      child = Text("starting ${widget.future.currentEpsilon()}...");
    }

    if (!widget.future.done() && !widget.future.started()) {
      widget.future.start();
    }

    if (widget.future.done()) {
      developer.log("SVG .. ${widget.future.result().length}");
      child = SvgPicture.string(
        widget.future.result(),
        width: 600,
        height: 150,
      );
    }
    return SizedBox(width: 600.0, child: Column(children: [child!]));
  }
}

class Renderings {
  final FutureRendering track;
  FutureRendering waypoints;
  Renderings({required this.track, required this.waypoints});
}

class SegmentStack extends StatefulWidget {
  final Renderings renderings;

  const SegmentStack({super.key, required this.renderings});

  @override
  State<SegmentStack> createState() => _SegmentStackState();
}

class _SegmentStackState extends State<SegmentStack> {
  @override
  Widget build(BuildContext context) {
    developer.log("build2 ${widget.renderings.track.currentEpsilon}...");
    return Stack(
      children: <Widget>[
        TrackWidget(future: widget.renderings.track),
        TrackWidget(future: widget.renderings.waypoints),
      ],
    );
  }
}
