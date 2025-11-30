import 'dart:developer' as developer;
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/static_svg_widget.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class WheelScreenWidget extends StatefulWidget {
  const WheelScreenWidget({super.key});
  @override
  State<WheelScreenWidget> createState() => _WheelScreenWidgetState();
}

class _WheelScreenWidgetState extends State<WheelScreenWidget> {
  String svg = "";
  @override
  void initState() {
    super.initState();
    File svgFile = File('/tmp/watch.svg');
    svg = svgFile.readAsStringSync();
    developer.log("initState");
  }

  @override
  Widget build(BuildContext ctx) {
    if (svg.isEmpty) {
      return Text("building");
    }
    SvgRootElement svgRootElement = parseSvg(svg);
    return StaticSvgWidget(svgRootElement: svgRootElement);
  }
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Card infoCard = Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.all(1), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: Text("infocard"),
      ),
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Wheel')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [WheelScreenWidget(), SizedBox(height: 150), UserStepsSliderProvider(),infoCard],
          ),
        ),
      ),
    );
  }
}
