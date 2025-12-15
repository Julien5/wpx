import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class WaypointsTableWidget extends StatelessWidget {
  final InputType kind;

  const WaypointsTableWidget({super.key, required this.kind});

  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  Widget buildData(List<Waypoint> waypoints) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No waypoints"));
    }
    return DataTable(
      // 1. Define the Columns
      columns: const <DataColumn>[
        DataColumn(
          label: Text('', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(
          label: Text('km', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(
          label: Text('GPX', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: false,
        ),
      ],
      rows:
          waypoints.map((w) {
            final formattedDistance = _formatDistance(w.info!.distance);
            final name = w.info!.name;
            final gpxName = w.info!.gpxName;

            return DataRow(
              cells: <DataCell>[
                DataCell(Text(name)),
                DataCell(Text(formattedDistance)),
                DataCell(
                  SizedBox(
                    width: 150, // Fixed width for the Name column
                    child: Text(
                      style: TextStyle(fontFamily: "mono"),
                      gpxName,
                      overflow:
                          TextOverflow.ellipsis, // Handle overflow gracefully
                    ),
                  ),
                ),
              ],
            );
          }).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<SegmentModel>(
      builder: (context, model, child) {
        Set<InputType> kinds = {kind};
        var waypoints = model.someWaypoints(kinds);
        return SingleChildScrollView(
          scrollDirection: Axis.vertical,
          child: buildData(waypoints),
        );
      },
    );
  }
}
