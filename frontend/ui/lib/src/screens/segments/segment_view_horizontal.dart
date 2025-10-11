import 'package:flutter/material.dart';
import 'profileview.dart';

class SegmentViewHorizontal extends StatelessWidget {
  const SegmentViewHorizontal({super.key});


  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
         double profileHeight = constraints.maxWidth * (285.0 / 1000);
        Widget profile = ProfileStack(profileHeight: profileHeight);
        return Center(child:profile);
      },
    );
  }
}
