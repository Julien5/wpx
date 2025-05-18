import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/globalfrontend.dart';
import 'package:ui/src/segment_widget.dart';
import 'package:ui/src/rust/api/frontend.dart';
import 'package:ui/src/counter.dart';

class SegmentsWidget extends StatefulWidget {
  const SegmentsWidget({super.key});

  @override
  State<SegmentsWidget> createState() => SegmentsWidgetState();
}

class SegmentsWidgetState extends State<SegmentsWidget> {
  List<SegmentWidget>? segmentWidgets;
  double currentDelta = 0;

  @override
  void initState() {
    super.initState();
    _loadSegments();
  }

  void _loadSegments() async {
    Frontend f = GlobalFrontend().frontend();
    List<FSegment> ret = await f.segments();
    developer.log("found ${ret.length} segments");
    setState(() {
      if (segmentWidgets==null || segmentWidgets!.length != ret.length) {
        segmentWidgets = [];
        for (var i = 0; i < ret.length; i++) {
          FSegment s = ret.elementAt(i);
          segmentWidgets!.add(SegmentWidget(segment: s, delta: currentDelta));
        }
      }
    });
  }

  void makeMorePoints() {
    Frontend f = GlobalFrontend().frontend();
    f.changeParameter(eps: -10.0);
    developer.log("makeMorePoints on ${segmentWidgets!.length} widgets");
    setState(() {
      for (var i = 0; i < segmentWidgets!.length; i++) {
        segmentWidgets!.elementAt(i).update();
      }
    });
  }

  void makeLessPoints() {
    Frontend f = GlobalFrontend().frontend();
    f.changeParameter(eps: 10.0);
    setState(() {
      for (var i = 0; i < segmentWidgets!.length; i++) {
        segmentWidgets!.elementAt(i).update();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    developer.log("SegmentsWidgetState build");
    if (segmentWidgets == null) {
      return Text("segments is null");
    }
    if (segmentWidgets!.isEmpty) {
      return Text("segments is empty");
    }

    return Column(
      children: [
        PressButton(label: "more", onCounterPressed: makeMorePoints),
        PressButton(label: "less", onCounterPressed: makeLessPoints),
        Expanded(
          child: ListView.separated(
            itemCount: segmentWidgets!.length,
            scrollDirection: Axis.vertical,
            separatorBuilder: (BuildContext context, int index) => const Divider(),
            itemBuilder: (context,index) {return segmentWidgets!.elementAt(index);},
          ),
        ),
      ],
    );
  }
}
