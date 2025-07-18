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
      dataRowMinHeight: 25,
      dataRowMaxHeight: 25,
      columns: const [
        DataColumn(label: Text('Name')),
        DataColumn(label: Text('KM')),
        DataColumn(label: Text('Elevation')),
        DataColumn(label: Text('Distance')),
        DataColumn(label: Text('Gain')),
        DataColumn(label: Text('Slope')),
        DataColumn(label: Text('Time')),
      ],
      rows:
          local.map((waypoint) {
            var dt = DateTime.parse(waypoint.time);
            return DataRow(
              cells: [
                DataCell(Text(waypoint.name.trim())),
                DataCell(
                  Text("${(waypoint.distance / 1000).toStringAsFixed(1)} km"),
                ), // Distance
                DataCell(
                  Text("${waypoint.elevation.floor().toStringAsFixed(0)} m"),
                ), // Elevation
                DataCell(
                  Text(
                    "${(waypoint.interDistance / 1000).toStringAsFixed(1)} km",
                  ),
                ),
                DataCell(
                  Text("${(waypoint.interElevationGain).toStringAsFixed(0)} m"),
                ),
                DataCell(Text("${(waypoint.interSlope).toStringAsFixed(1)} %")),
                DataCell(
                  Text(
                    DateFormat('HH:mm').format(dt),
                  ), // Format the DateTime object
                ),
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
