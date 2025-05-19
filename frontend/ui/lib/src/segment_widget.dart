import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/profile_widget.dart';
import 'package:ui/src/rust/api/frontend.dart';

class SegmentWidget extends StatelessWidget {
  final GlobalKey<TrackWidgetState> waypointsKey =
      GlobalKey<TrackWidgetState>();

  final FSegment segment;
  SegmentWidget({super.key, required this.segment});

  void update() {
    if (waypointsKey.currentState == null) {
      developer.log("current state is null");
      return;
    }
    developer.log("current state is NOT null");
    waypointsKey.currentState?.load();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16.0),
          decoration: BoxDecoration(
            border: Border.all(color: Colors.blue, width: 1.0),
            borderRadius: BorderRadius.circular(8.0),
          ),
          child: SegmentStack(segment: segment, waypointsKey: waypointsKey),
        ),
      ],
    );
  }
}
