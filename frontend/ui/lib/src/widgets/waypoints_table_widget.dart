import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart';

class DesktopTable extends StatefulWidget {
  final List<Waypoint> waypoints;
  final bool editControls;

  const DesktopTable({
    super.key,
    required this.waypoints,
    required this.editControls,
  });

  @override
  State<DesktopTable> createState() => _DesktopTableState();
}

class _DesktopTableState extends State<DesktopTable> {
  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  String _formatTime(String isoTime) {
    DateTime time = parseDateTime(isoTime);
    return DateFormat('HH:mm').format(time);
  }

  void makeControlAtWaypoint(Waypoint waypoint, bool on) {
    SegmentModel segment = Provider.of(context, listen: false);
    segment.makeControlAtWaypoint(waypoint, on);
    KindsModel kinds = Provider.of(context, listen: false);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      // KindsModel wants to know how many controls there are.
      kinds.updateStatistics(segment.statistics());
    });
  }

  Widget buildData(List<Waypoint> waypoints) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No waypoints"));
    }
    debugPrint("build table with ${waypoints.length} waypoints");
    return DataTable(
      // 1. Define the Columns
      columns: <DataColumn>[
        const DataColumn(
          label: Text('km', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        const DataColumn(
          label: Text('time', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        const DataColumn(label: Text(""), numeric: false),
        if (widget.editControls)
          const DataColumn(
            label: Text(
              'control',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            numeric: false,
          ),
      ],
      rows:
          waypoints.map((w) {
            final formattedDistance = _formatDistance(w.info!.distance);
            final time = _formatTime(w.info!.time);
            final description = joinNonEmpty([w.name, w.description]);
            final isGpxWaypoint = w.origin == Kind.gpxWaypoints;
            final isControl = w.origin == Kind.controls;
            final showCheckbox = isControl || isGpxWaypoint;
            final checkBoxValue = isControl;
            return DataRow(
              cells: <DataCell>[
                DataCell(Text(formattedDistance)),
                DataCell(Text(time)),
                DataCell(Text(description)),
                if (widget.editControls)
                  DataCell(
                    showCheckbox
                        ? Checkbox(
                          value: checkBoxValue,
                          onChanged: (bool? value) {
                            setState(() {
                              if (value != null) {
                                makeControlAtWaypoint(w, value);
                              }
                            });
                          },
                        )
                        : const SizedBox.shrink(),
                  ),
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
      child: buildData(widget.waypoints),
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
