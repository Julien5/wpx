import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:ui/src/profile_widget.dart';

class SegmentWidget extends StatefulWidget {
  final Renderings renderings;
  const SegmentWidget({super.key, required this.renderings});

  @override
  State<SegmentWidget> createState() => _SegmentWidgetState();
}

class _SegmentWidgetState extends State<SegmentWidget> {
  @override
  Widget build(BuildContext context) {
    developer.log("build4 ${widget.renderings.track.currentEpsilon}...");
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16.0),
          decoration: BoxDecoration(
            border: Border.all(color: Colors.blue, width: 1.0),
            borderRadius: BorderRadius.circular(8.0),
          ),
          child: SegmentStack(renderings: widget.renderings),
        ),
      ],
    );
  }
}
