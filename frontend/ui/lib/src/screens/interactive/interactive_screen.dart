//import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';

class InteractiveMapView extends StatelessWidget {
  const InteractiveMapView({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel rootModel = Provider.of<RootModel>(context);

    return LayoutBuilder(
      builder: (context, constraints) {
        return Text("hi");
      },
    );
  }
}

class InteractiveConsumer extends StatelessWidget {
  const InteractiveConsumer({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(children: [Expanded(child: InteractiveMapView())]),
      ),
    );
  }
}

class InteractiveScreen extends StatelessWidget {
  const InteractiveScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(title: const Text('Map')),
      body: InteractiveConsumer(),
    );
  }
}
