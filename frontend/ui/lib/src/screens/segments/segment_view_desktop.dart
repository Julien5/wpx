import 'package:flutter/material.dart';
import 'mapview.dart';
import 'profileview.dart';
import 'waypoints_widget.dart';

class SegmentViewDesktop extends StatelessWidget {
  const SegmentViewDesktop({super.key});

  Widget wideView() {
    var table = WayPointsWidget();
    var hspace = const Expanded(child: SizedBox(width: 10));
    var map = MapConsumer();
    var row = Expanded(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [hspace, map, hspace, table, hspace],
      ),
    );
    return row;
  }

  Widget thinView() {
    var row = ConstrainedBox(
      constraints: const BoxConstraints(maxHeight: 400),
      child: MapScrollWidget(),
    );
    return Expanded(child:row);
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Widget profile = ProfileStack(profileHeight: 285,);
        Widget? maprow;
        
        if (constraints.maxWidth > 1000) {
          maprow = wideView();
        } else {
          maprow = thinView();
        }
        var hline = const Divider(
          height: 1, // Thickness of the divider
          color: Colors.grey, // Light stroke color
        );
        return Column(children: [profile, hline, maprow]);
      },
    );
  }
}
