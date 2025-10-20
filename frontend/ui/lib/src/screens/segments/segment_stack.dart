import 'package:flutter/material.dart';
import 'package:ui/src/screens/segments/segment_view_desktop.dart';
import 'package:ui/src/screens/segments/segment_view_horizontal.dart';
import 'package:ui/src/screens/segments/vertical/segment_view_vertical.dart';

enum ScreenOrientation { desktop, landscape, portrait }

class SegmentView extends StatelessWidget {
  final ScreenOrientation screenOrientation;
  const SegmentView({super.key, required this.screenOrientation});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        switch (screenOrientation) {
          case ScreenOrientation.desktop:
            return SegmentViewDesktop();
          case ScreenOrientation.portrait:
            return SegmentViewVertical();
          case ScreenOrientation.landscape:
            return SegmentViewHorizontal();
        }
      },
    );
  }
}
