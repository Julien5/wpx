import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';

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
    if (widget.trackData == TrackData.track) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        loadBackground(context); // Call load() in the next frame
      });
    } else {
      
    }

  }

  void loadBackground(BuildContext context) async {
    BackendModel backend = BackendModel.of(context);
    String d = await backend.renderSegmentTrack(widget.segment);
      
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
    if (widget.trackData == TrackData.waypoints) {
      svgData=BackendModel.of(context).renderSegmentWaypoints(widget.segment);
    }
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

  const SegmentStack({
    super.key,
    required this.segment,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        TrackWidget(segment: segment, trackData: TrackData.track),
        TrackWidget(segment: segment, trackData: TrackData.waypoints),
      ],
    );
  }
}
