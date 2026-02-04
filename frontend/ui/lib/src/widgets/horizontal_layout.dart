import 'package:flutter/material.dart';

class HorizontalLayout extends StatelessWidget {
  final Widget topRow;
  final List<Widget> midChildren;
  const HorizontalLayout({
    super.key,
    required this.topRow,
    required this.midChildren,
  });

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (context, constraints) {
        const double height = 400;
        const double midSpace = 30;
        final double availWidth = 800 + midSpace;

        // we should take the constraints into account.

        List<Widget> children = [
          ConstrainedBox(
            constraints: BoxConstraints(maxHeight: 400, maxWidth: 400),
            child: topRow,
          ),
          SizedBox(width: midSpace),
          ConstrainedBox(
            constraints: BoxConstraints(maxHeight: 380, maxWidth: 400),
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
              constraints: BoxConstraints(maxHeight: height),
              child: SingleChildScrollView(
                scrollDirection: Axis.vertical,
                child: ConstrainedBox(
                  constraints: BoxConstraints(maxWidth: availWidth),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: children,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
