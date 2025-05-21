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
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.future,
      builder: (context, _) {
        return buildWorker(context);
      },
    );
  }

  Widget buildWorker(BuildContext context) {
    Widget child;
    if (!widget.future.done()) {
      child = Text("loading ${widget.future.currentEpsilon}...");
    } else {
      child = SvgPicture.string(
        widget.future.result(),
        width: 600,
        height: 150,
      );
    }
    return SizedBox(width: 600.0, child: Column(children: [child]));
  }
}

class Renderings {
  final FutureRendering track;
  final FutureRendering waypoints;
  const Renderings({required this.track, required this.waypoints});
}

class SegmentStack extends StatelessWidget {
  final Renderings renderings;

  const SegmentStack({super.key, required this.renderings});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        TrackWidget(future: renderings.track),
        TrackWidget(future: renderings.waypoints),
      ],
    );
  }
}
