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
  List<SegmentWidget> segments = [];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      updateSegments();
    });
  }

  void updateSegments() {
    BackendModel backend = BackendModel.of(context);
    var S = backend.segments();
    if (S.length != segments.length) {
      segments.clear();
      for (var segment in S) {
        var track = backend.renderSegmentTrack(segment);
        var wp = backend.renderSegmentWaypoints(segment);
        track.start();
        segments.add(
          SegmentWidget(renderings: Renderings(track: track, waypoints: wp)),
        );
      }
    } else {
      for (int i = 0; i < S.length; i++) {
        //var wp = backend.renderSegmentWaypoints(S[i]);
        // this is where we need something like: segments[i].update();
        segments[i].renderings.waypoints.reset();
      }
    }
    developer.log("made ${segments.length} segments");
    setState(() {});
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
    developer.log("[segments] [build] delta=${backend.epsilon()}");
    if (segments.isEmpty) {
      return Text("segments is empty");
    }
    return Column(
      children: [
        PressButton(label: "more", onCounterPressed: makeMorePoints),
        PressButton(label: "less", onCounterPressed: makeLessPoints),
        Expanded(child: ListView(children: segments)),
      ],
    );
  }
}
