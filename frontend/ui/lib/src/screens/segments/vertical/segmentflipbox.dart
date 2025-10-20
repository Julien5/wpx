import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';

import '../mapview.dart';
import '../profileview.dart';

class SegmentFlipBox extends StatefulWidget {
  const SegmentFlipBox({super.key});

  @override
  State<SegmentFlipBox> createState() => _SegmentFlipBoxState();
}

class _SegmentFlipBoxState extends State<SegmentFlipBox> {
  static bool flipped = false;
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        double margin = 10;
        // profile height so that the profile width is quite within the screen
        double profileHeight = (285 / 1000) * (constraints.maxWidth - margin);
        double innerHeight = min(
          constraints.maxWidth - margin,
          profileHeight * 2,
        );
        double innerWidth = constraints.maxWidth-4*margin;

        developer.log("flipped= $flipped");

        BoxConstraints innerbox = BoxConstraints(
          maxWidth: innerWidth,
          maxHeight: innerHeight,
        );

        BoxConstraints outerbox = BoxConstraints(
          minWidth: innerWidth + margin,
          maxWidth: innerWidth + 2 * margin,
          minHeight: innerHeight + margin,
          maxHeight: innerHeight + 2 * margin,
        );
        Widget widget =
            !flipped
                ? MapConsumer()
                : ProfileStack(profileHeight: profileHeight);
        Widget innerWidget = ConstrainedBox(
          constraints: innerbox,
          child: Center(child: widget),
        );
        Widget outerwidget = ConstrainedBox(
          constraints: outerbox,
          child: Center(
            child: Stack(
              children: [
                Container(color: Colors.white),
                innerWidget,
                Positioned(
                  top: 10,
                  right: 10,
                  child: ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      padding: EdgeInsets.zero,
                      minimumSize: Size(40, 40),
                    ),
                    onPressed: () {
                      setState(() {
                        flipped = !flipped;
                      });
                    },
                    child: Icon(Icons.show_chart_rounded),
                  ),
                ),
              ],
            ),
          ),
        );
        return DecoratedBox(
          decoration: BoxDecoration(
            border: Border.all(color: Colors.blue, width: 2),
            borderRadius: BorderRadius.circular(margin),
            boxShadow: [
              BoxShadow(
                color: Colors.grey.withAlpha(255),
                blurRadius: margin, 
                spreadRadius: margin/2,
                offset: Offset(
                  0,
                  6.0,
                ), // How far the shadow is offset from the box
              ),
            ],
          ),
          child: outerwidget,
        );
      },
    );
  }
}
