import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/segment_stack.dart';

class SegmentsWidget extends StatefulWidget {
  const SegmentsWidget({super.key});

  @override
  State<SegmentsWidget> createState() => SegmentsWidgetState();
}

class SegmentsWidgetState extends State<SegmentsWidget> {
  final List<RenderingsProvider> _segments = [];
  SegmentsProvider? provider;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      updateSegments();
    });
  }

  void postInit() {
    if (provider != null) {
      return;
    }
    provider = SegmentsProvider.of(context);
    provider!.notifier.addListener(() {
      onSegmentsChanged();
    });
  }

  @override
  void dispose() {
    provider!.notifier.removeListener(() {
      onSegmentsChanged();
    });
    super.dispose();
  }

  void onSegmentsChanged() {
    updateSegments();
  }

  // called immediately after initState() with safe context.
  @override
  void didChangeDependencies() {
    developer.log("[didChangeDependencies]");
    postInit();
    super.didChangeDependencies();
  }

  void updateSegments() {
    var segmentsProvider = SegmentsProvider.of(context);
    var S = segmentsProvider.segments();
    if (S.length != _segments.length) {
      _segments.clear();
      for (var segment in S) {
        var provider = RenderingsProvider(
          renderings: segmentsProvider.createRenderings(segment),
          child: SegmentStack(),
        );
        _segments.add(provider);
      }
    } else {
      for (var segment in _segments) {
        segment.renderings.waypoints.reset();
      }
    }
    setState(() {});
  }

  void makeMorePoints() {
    var backend = SegmentsProvider.of(context);
    backend.decrementDelta();
  }

  void makeLessPoints() {
    var backend = SegmentsProvider.of(context);
    backend.incrementDelta();
  }

  @override
  Widget build(BuildContext context) {
    developer.log("[segments] [build] #segments=${_segments.length}");
    if (_segments.isEmpty) {
      return Text("segments is empty");
    }
    return Column(
      children: [
        Row(
          children: [
            PressButton(label: "more", onCounterPressed: makeMorePoints),
            PressButton(label: "less", onCounterPressed: makeLessPoints),
          ],
        ),
        Expanded(
          child: ListView.separated(
            itemCount: _segments.length,
            separatorBuilder: (context, index) => const Divider(),
            itemBuilder: (context, index) {
              return _segments[index];
            },
          ),
        ),
      ],
    );
  }
}
