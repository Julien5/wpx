import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class WayPointsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  final Segment? segment;
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
    List<WayPoint> all = widget.segmentsProvider!.waypoints();

    List<WayPoint> local = [];
    for (WayPoint waypoint in all) {
      if (widget.segment!.showsWaypoint(wp: waypoint)) {
        local.add(waypoint);
      }
    }

    developer.log("[WayPointsViewState] [build] #_waypoints=${local.length}");

    if (local.isEmpty) {
      return const Center(child: Text("No waypoints available"));
    }

    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: DataTable(
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
              var dt = DateTime.fromMillisecondsSinceEpoch(
                waypoint.time.toInt() * 1000,
              );
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
                    Text(
                      "${(waypoint.interElevationGain).toStringAsFixed(0)} m",
                    ),
                  ),
                  DataCell(
                    Text("${(waypoint.interSlope).toStringAsFixed(1)} %"),
                  ),
                  DataCell(
                    Text(
                      DateFormat('HH:mm').format(dt),
                    ), // Format the DateTime object
                  ),
                ],
              );
            }).toList(),
      ),
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
        // TODO: use wp to get the current segment and use it to get the waypoints.
        var segment = wp.segment;
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                Expanded(
                  child: WayPointsView(
                    segmentsProvider: segmentsProvider,
                    segment: segment,
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
