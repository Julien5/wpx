import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/adaptive_layout.dart';
import 'package:wpx/src/widgets/segmentgraphics.dart';
import 'package:wpx/src/widgets/waypoints_table_widget.dart';

class _TextWidget extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    Kinds control = [Kind.controls];
    SegmentModel model = Provider.of<SegmentModel>(context);
    List<Waypoint> waypoints = model.someWaypoints(control);
    String text =
        waypoints.isEmpty ? "no controls" : "${waypoints.length} controls";
    return Center(child: Text(text));
  }
}

class _Table extends StatefulWidget {
  @override
  State<_Table> createState() => _TableState();
}

class _TableState extends State<_Table> {
  DesktopTable? table;
  @override
  Widget build(BuildContext context) {
    FutureRenderer renderer = Provider.of<FutureRenderer>(context);
    RenderOutput? renderOutput = renderer.renderOutput(RenderFunction.profile);
    if (renderOutput != null) {
      // remove osm points.
      List<Waypoint> noOsm =
          renderOutput.waypoints
              .where(
                (point) =>
                    point.origin == Kind.gpxWaypoints ||
                    point.origin == Kind.controls,
              )
              .toList();
      List<Waypoint> waypoints = decimate(
        waypoints: noOsm,
        segment: renderer.getSegment(),
        n: BigInt.from(renderOutput.waypoints.length),
      );
      table = DesktopTable(waypoints: waypoints, editControls: true);
    }
    if (table == null) {
      return Text("no waypoints");
    }
    return table!;
  }
}

class _ControlsScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    Kinds control = [Kind.controls];
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () => gotoOverview(ctx),
        ),
        title: const Text('Control Points'),
      ),
      body: MobileScaffoldBody(
        topRow: TrackGraphicsRow(kinds: control),
        midColumn: MidColumn(
          children: [_TextWidget(), Expanded(child: _Table())],
        ),
        screenFocus: ScreenFocus.controls,
        clients: [
          RenderFunction.profile,
          RenderFunction.map,
          RenderFunction.wheel,
        ],
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
