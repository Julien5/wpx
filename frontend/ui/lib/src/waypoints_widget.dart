import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class WayPointsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  final bridge.Segment? segment;
  const WayPointsView({super.key, this.segmentsProvider, this.segment});

  @override
  State<WayPointsView> createState() => WayPointsViewState();
}

class WayPointsViewState extends State<WayPointsView> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    List<bridge.Step> all = widget.segmentsProvider!.waypoints();

    List<bridge.Step> local = [];
    for (bridge.Step waypoint in all) {
      if (widget.segment!.showsWaypoint(wp: waypoint)) {
        local.add(waypoint);
      }
    }

    developer.log("[WayPointsViewState] [build] #_waypoints=${local.length}");

    if (local.isEmpty) {
      return const Center(child: Text("No waypoints available"));
    }

    return DataTable(
      columnSpacing: 20,
      dataRowMinHeight: 25,
      dataRowMaxHeight: 25,
      columns: const [
        DataColumn(label: Text('KM'), numeric: true),
        DataColumn(label: Text('Time'), numeric: true),
        DataColumn(label: Text('Dist.'), numeric: true),
        DataColumn(label: Text('Elev.'), numeric: true),
        DataColumn(label: Text('Slope'), numeric: true),
      ],
      rows:
          local.map((waypoint) {
            var dt = DateTime.parse(waypoint.time);
            var km = waypoint.distance / 1000;
            var ikm = waypoint.interDistance / 1000;
            var egain = waypoint.interElevationGain;
            var slope = waypoint.interSlope;
            return DataRow(
              cells: [
                DataCell(Text("${km.toStringAsFixed(0)}")),
                DataCell(Text(DateFormat('HH:mm').format(dt))),
                DataCell(Text("${ikm.toStringAsFixed(1)} km")),
                DataCell(Text("${egain.toStringAsFixed(0)} m")),
                DataCell(Text("${slope.toStringAsFixed(1)} %")),
              ],
            );
          }).toList(),
    );
  }
}

class WayPointsConsumer extends StatelessWidget {
  const WayPointsConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        var wp = Provider.of<WaypointsRenderer>(context, listen: false);
        var segment = wp.segment;
        return Center(
          child: WayPointsView(
            segmentsProvider: segmentsProvider,
            segment: segment,
          ),
        );
      },
    );
  }
}
