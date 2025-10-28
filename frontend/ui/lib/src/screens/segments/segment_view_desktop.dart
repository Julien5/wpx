import 'package:flutter/material.dart';
import 'package:ui/src/widgets/indicatorselector.dart';
import 'package:ui/src/widgets/parameterslider.dart';
import 'mapview.dart';
import 'profileview.dart';
import 'waypoints_widget.dart';

class SegmentViewDesktop extends StatelessWidget {
  const SegmentViewDesktop({super.key});

  Widget wideView() {
    var hspace = const Expanded(child: SizedBox(width: 10));
    var vspace = const Expanded(child: SizedBox(height: 10));
    var settings = Column(
      children: [vspace,ElevationIndicatorChooser(), ParameterSliderProvider(),vspace],
    );
    
    var map = MapConsumer();
    var row = Expanded(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [hspace, map, hspace, settings, hspace],
      ),
    );
    return row;
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Widget profile = ProfileStack(profileHeight: 285);
        Widget? maprow = wideView();
        var hline = const Divider(
          height: 1, // Thickness of the divider
          color: Colors.grey, // Light stroke color
        );
        return Column(children: [profile, hline, maprow]);
      },
    );
  }
}
