import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/utils/utils.dart';

class DesktopTable extends StatelessWidget {
  final List<Waypoint> waypoints;

  const DesktopTable({super.key, required this.waypoints});

  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  String _formatTime(String isoTime) {
    DateTime time = parseDateTime(isoTime);
    return DateFormat('HH:mm').format(time);
  }

  Widget buildData(List<Waypoint> waypoints) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No waypoints"));
    }

    debugPrint("build table with ${waypoints.length} waypoints");
    return DataTable(
      // 1. Define the Columns
      columns: const <DataColumn>[
        DataColumn(
          label: Text('km', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(
          label: Text('time', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(label: Text(""), numeric: false),
      ],
      rows:
          waypoints.map((w) {
            final formattedDistance = _formatDistance(w.info!.distance);
            final time = _formatTime(w.info!.time);
            final description = joinNonEmpty([w.name, w.description]);
            return DataRow(
              cells: <DataCell>[
                DataCell(Text(formattedDistance)),
                DataCell(Text(time)),
                DataCell(Text(description)),
              ],
            );
          }).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context);
    debugPrint("build table for segment ${model.segment.id()}");
    context.watch<ParameterModel>();
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: buildData(waypoints),
    );
  }
}

class GPXTable extends StatelessWidget {
  final Set<Kind> kinds;

  const GPXTable({super.key, required this.kinds});

  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  Widget buildData(List<Waypoint> waypoints) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No point."));
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
            // split at anything but letters and numbers
            final name = w.info!.name.split(RegExp(r'[^a-zA-Z0-9]')).first;
            final gpxName = w.info!.gpxName;

            return DataRow(
              cells: <DataCell>[
                DataCell(Text(name, textAlign: TextAlign.left, maxLines: 1)),
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
    SegmentModel model = Provider.of<SegmentModel>(context);
    context.watch<ParameterModel>();
    var waypoints = model.someWaypoints(kinds);
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: buildData(waypoints),
    );
  }
}
