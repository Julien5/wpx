import 'package:flutter/material.dart';
import 'package:ui/src/widgets/userstepsslider.dart';
import '../../../widgets/indicatorselector.dart';
import 'segmentflipbox.dart';

class SegmentViewVertical extends StatelessWidget {
  const SegmentViewVertical({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        // Center the content vertically
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          SegmentFlipBox(),
          SizedBox(height: 50),
          ElevationIndicatorChooser(),
          UserStepsSliderProvider(),
        ],
      ),
    );
  }
}
