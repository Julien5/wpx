import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/globalfrontend.dart';
import 'dart:developer' as developer;

import 'package:ui/src/rust/api/frontend.dart';

class TrackWidget extends StatefulWidget {
  final FSegment segment;
  const TrackWidget({super.key, required this.segment});
  @override
  State<TrackWidget> createState() => TrackWidgetState();
}

class TrackWidgetState extends State<TrackWidget> {
  String? svgData;
  @override
  void initState() {
    super.initState();
    loadTrack();
  }

  void loadTrack() async {
    if (!mounted) {
      return;
    }
    assert(GlobalFrontend().loaded());
    Frontend f = GlobalFrontend().frontend();
    String d = await f.renderSegmentTrack(segment: widget.segment);
    developer.log("found ${d.length} svg track bytes");
    if (!mounted) {
      return;
    }
    setState(() {
      svgData = d;
    });
  }

  @override
  Widget build(BuildContext context) {
    Widget child;
    if (svgData == null) {
      child = Text("loading track...");
    } else {
      child = SvgPicture.string(svgData!, width: 600, height: 150);
    }
    return SizedBox(width: 600.0, child: Column(children: [child]));
  }
}

class WaypointsWidget extends StatefulWidget {
  final FSegment segment;
  const WaypointsWidget({super.key, required this.segment});

  @override
  State<WaypointsWidget> createState() => WaypointsWidgetState();
}

class WaypointsWidgetState extends State<WaypointsWidget> {
  String? svgData;
  @override
  void initState() {
    super.initState();
    loadWaypoints();
  }

  void loadWaypoints() async {
    if (!mounted) {
      return;
    }
    assert(GlobalFrontend().loaded());
    Frontend f = GlobalFrontend().frontend();
    String d = await f.renderSegmentWaypoints(segment: widget.segment);
    if (!mounted) {
      developer.log("unmounted");
      return;
    }
    developer.log("found ${d.length} svg waypoints bytes");
    setState(() {
      svgData = d;
    });
  }

  @override
  Widget build(BuildContext context) {
    Widget child;
    if (svgData == null) {
      child = Text("loading.");
    } else {
      child = SvgPicture.string(svgData!, width: 600, height: 150);
    }
    return SizedBox(width: 600.0, child: Column(children: [child]));
  }
}

class SegmentKey {
  final FSegment segment;
  final double delta;
  const SegmentKey(this.segment, this.delta);
}

class SegmentStack extends StatelessWidget {
  final FSegment segment;
  final GlobalKey<WaypointsWidgetState> waypointsKey;

  const SegmentStack({
    super.key,
    required this.segment,
    required this.waypointsKey,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        TrackWidget(segment: segment),
        WaypointsWidget(key: waypointsKey, segment: segment),
      ],
    );
  }
}
