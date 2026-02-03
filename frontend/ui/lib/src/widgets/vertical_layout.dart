import 'package:flutter/material.dart';

class VerticalLayout extends StatelessWidget {
  final Widget topRow;
  final List<Widget> midChildren;
  const VerticalLayout({
    super.key,
    required this.topRow,
    required this.midChildren,
  });

  @override
  Widget build(BuildContext ctx) {
    const double colWidth = 400;

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 200),
        child: topRow,
      ),
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 380),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          mainAxisAlignment: MainAxisAlignment.start,
          children: midChildren,
        ),
      ),
    ];

    return Align(
      alignment: Alignment.topCenter,
      child: Padding(
        padding: EdgeInsets.fromLTRB(10, 0, 10, 0),
        child: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: colWidth),
          child: SingleChildScrollView(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.start,
              children: children,
            ),
          ),
        ),
      ),
    );
  }
}
