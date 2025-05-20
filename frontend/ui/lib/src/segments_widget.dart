import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/segment_widget.dart';

class SegmentsWidget extends StatefulWidget {
  const SegmentsWidget({super.key});

  @override
  State<SegmentsWidget> createState() => SegmentsWidgetState();
}

class SegmentsWidgetState extends State<SegmentsWidget> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    assert(BackendModel.maybeOf(context) != null);
    FrontendNotifier notifier = BackendModel.of(context).notifier;
    BackendModel backend = BackendModel.of(context);
    return ListenableBuilder(
      listenable: notifier,
      builder: (context,_) {
        return buildWorker(context, backend);
      },);
  }

  void makeMorePoints() {
    BackendModel backend = BackendModel.of(context);
    backend.decrementDelta();
    developer.log("delta=${backend.epsilon()}");
  }

  void makeLessPoints() {
    BackendModel backend = BackendModel.of(context);
    backend.incrementDelta();
    developer.log("delta=${backend.epsilon()}");
  }

  Widget buildWorker(BuildContext context, BackendModel backend) {
    developer.log("delta=${backend.epsilon()}");
    var segments = backend.segments();
    if (segments.isEmpty) {
      return Text("segments is empty");
    }
    List<Widget> W = [];
    for (var segment in segments) {
      W.add(SegmentWidget(segment: segment));
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
