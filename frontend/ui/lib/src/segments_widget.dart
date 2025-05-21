import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/profile_widget.dart';
import 'package:ui/src/segment_widget.dart';

class SegmentsWidget extends StatefulWidget {
  const SegmentsWidget({super.key});

  @override
  State<SegmentsWidget> createState() => SegmentsWidgetState();
}

class SegmentsWidgetState extends State<SegmentsWidget> {
  List<Renderings> segments = [];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      updateSegments();
    });
  }

  void updateSegments() {
    segments.clear();
    BackendModel backend = BackendModel.of(context);
    var S = backend.segments();
    for (var segment in S) {
      var track = backend.renderSegmentTrack(segment);
      var wp = backend.renderSegmentWaypoints(segment);
      segments.add(Renderings(track: track, waypoints: wp));
    }
    developer.log("made ${segments.length} segments");
    setState(() {
    });
  }

  void makeMorePoints() {
    BackendModel backend = BackendModel.of(context);
    backend.decrementDelta();
    developer.log("delta=${backend.epsilon()}");
    updateSegments();
  }

  void makeLessPoints() {
    BackendModel backend = BackendModel.of(context);
    backend.incrementDelta();
    developer.log("delta=${backend.epsilon()}");
    updateSegments();
  }

  @override
  Widget build(BuildContext context) {
    BackendModel backend = BackendModel.of(context);
    developer.log("[segments] [buildWorker] delta=${backend.epsilon()}");
    if (segments.isEmpty) {
      return Text("segments is empty");
    }
    List<Widget> W = [];
    for (var renderings in segments) {
      W.add(SegmentWidget(renderings: renderings));
    }

    return Column(
      children: [
        PressButton(label: "more", onCounterPressed: makeMorePoints),
        PressButton(label: "less", onCounterPressed: makeLessPoints),
        Expanded(child: ListView(children: W)),
      ],
    );
  }
}
