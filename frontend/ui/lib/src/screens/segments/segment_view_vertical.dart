import 'package:flutter/material.dart';
import 'mapview.dart';
import 'profileview.dart';

class SegmentViewVertical extends StatelessWidget {
  const SegmentViewVertical({super.key});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        double profileHeight = constraints.maxWidth * (285.0 / 400);
        double mapHeight = constraints.maxWidth;
        Widget profile = ProfileStack(profileHeight: profileHeight);
        Widget map = ConstrainedBox(
          constraints: BoxConstraints(maxHeight: mapHeight),
          child: MapConsumer(),
        );
        var hline = const Divider(
          height: 1, // Thickness of the divider
          color: Colors.grey, // Light stroke color
        );
        return Column(children: [Expanded(flex:285,child:profile), hline, Expanded(flex:400,child:map)]);
      },
    );
  }
}
