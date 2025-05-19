import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/globalfrontend.dart';
import 'dart:developer' as developer;

import 'package:ui/src/rust/api/frontend.dart';

enum TrackData { track, waypoints }

class TrackWidget extends StatefulWidget {
  final FSegment segment;
  final TrackData trackData;
  const TrackWidget({
    super.key,
    required this.segment,
    required this.trackData,
  });
  @override
  State<TrackWidget> createState() => TrackWidgetState();
}

class TrackWidgetState extends State<TrackWidget> {
  String? svgData;
  @override
  void initState() {
    super.initState();
    load();
  }

  void load() async {
    if (!mounted) {
      return;
    }
    assert(GlobalFrontend().loaded());
    Frontend f = GlobalFrontend().frontend();
    String d;
    if (widget.trackData == TrackData.track) {
      d = await f.renderSegmentTrack(segment: widget.segment);
    } else {
      d = await f.renderSegmentWaypoints(segment: widget.segment);
    }
    developer.log("found ${d.length} svg track bytes for ${widget.trackData}");
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
      child = Text("loading ${widget.trackData}...");
    } else {
      child = SvgPicture.string(svgData!, width: 600, height: 150);
    }
    return SizedBox(width: 600.0, child: Column(children: [child]));
  }
}

class SegmentStack extends StatelessWidget {
  final FSegment segment;
  final GlobalKey<TrackWidgetState> waypointsKey;

  const SegmentStack({
    super.key,
    required this.segment,
    required this.waypointsKey,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        TrackWidget(segment: segment, trackData: TrackData.track,),
        TrackWidget(key: waypointsKey, segment: segment, trackData: TrackData.waypoints,),
      ],
    );
  }
}
