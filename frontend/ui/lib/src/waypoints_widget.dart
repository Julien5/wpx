import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class WayPointsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  const WayPointsView({super.key, this.segmentsProvider});

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
    List<WayPoint> waypoints = widget.segmentsProvider!.waypoints();

    developer.log(
      "[WayPointsViewState] [build] #_waypoints=${waypoints!.length}",
    );

    if (waypoints.isEmpty) {
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
            waypoints.map((waypoint) {
              var dt = DateTime.fromMillisecondsSinceEpoch(
                waypoint.time * 1000,
              );
              return DataRow(
                cells: [
                  DataCell(Text(waypoint.name.trim())),
                  DataCell(
                    Text("${(waypoint.distance / 1000).toStringAsFixed(1)} km"),
                  ), // Distance
                  DataCell(
                    Text("${waypoint.elevation.toStringAsFixed(0)} m"),
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
                  DataCell(Text(dt.toIso8601String())),
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
        SegmentsProvider provider = Provider.of<SegmentsProvider>(
          context,
          listen: false,
        );

        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                Expanded(child: WayPointsView(segmentsProvider: provider)),
              ],
            ),
          ),
        );
      },
    );
  }
}
