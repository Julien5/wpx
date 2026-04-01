import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/widgets/horizontal_layout.dart';
import 'package:wpx/src/widgets/vertical_layout.dart';

class AdaptiveLayout extends StatelessWidget {
  final Widget topRow;
  final List<Widget> midChildren;
  const AdaptiveLayout({
    super.key,
    required this.topRow,
    required this.midChildren,
  });

  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    if (screen.mode == DisplayMode.vertical) {
      return VerticalLayout(topRow: topRow, midChildren: midChildren);
    }
    return HorizontalLayout(topRow: topRow, midChildren: midChildren);
  }
}
