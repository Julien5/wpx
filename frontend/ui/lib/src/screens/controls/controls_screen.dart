import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_table.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';

class _ButtonWidget extends StatelessWidget {
  void gotoTable(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => ControlsTableScreen(model: model),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    Widget tableButton = ElevatedButton(
      onPressed: () => gotoTable(context),
      child: const Text("Table"),
    );
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [SizedBox(height: 10), tableButton, SizedBox(height: 10)],
    );
  }
}

class _TextWidget extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    Set<Kind> control = {Kind.controls};
    SegmentModel model = Provider.of<SegmentModel>(context);
    List<Waypoint> waypoints = model.someWaypoints(control);
    String text =
        waypoints.isEmpty ? "no controls" : "${waypoints.length} controls";
    return Center(child: Text(text));
  }
}

class _ControlsScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    Set<Kind> control = {Kind.controls};
    return Scaffold(
      appBar: AppBar(title: const Text('Control Points')),
      body: AdaptiveLayout(
        topRow: TrackGraphicsRow(kinds: control),
        midChildren: [_TextWidget(), _ButtonWidget()],
      ),
    );
  }
}

class ControlsScreen extends StatelessWidget {
  const ControlsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return _ControlsScaffold();
  }
}
