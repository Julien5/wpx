import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/wheel/control_time_dialog.dart';
import 'package:wpx/src/screens/wheel/speed_dialog.dart';
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

Text makeTimeLabel(Waypoint w, FontWeight weight) {
  return Text(
    formatTime(parseDateTime(w.info!.time)),
    style: TextStyle(fontSize: 12, fontWeight: weight),
  );
}

class ControlEditTimeButton extends StatelessWidget {
  final Parameters parameters;
  final Waypoint? previousControl;
  final Waypoint? nextControl;
  final Waypoint currentControl;

  final Function(DateTime) onTimeChanged;
  const ControlEditTimeButton({
    super.key,
    required this.previousControl,
    this.nextControl,
    required this.currentControl,
    required this.onTimeChanged,
    required this.parameters,
  });

  @override
  Widget build(BuildContext context) {
    // we are not in acp mode
    FontWeight weight =
        currentControl.hasCustomTime ? FontWeight.bold : FontWeight.normal;
    return ElevatedButton(
      onPressed: () {
        openControlTimeDialog(
          context: context,
          parameters: parameters,
          previousControl: previousControl,
          nextControl: nextControl,
          currentControl: currentControl,
          onTimeChanged: onTimeChanged,
        );
      },
      style: ElevatedButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 4.0),
        minimumSize: const Size(0, 0),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      ),
      child: makeTimeLabel(currentControl, weight),
    );
  }
}

class _DesktopTableState extends State<DesktopTable> {
  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  void makeControlAtWaypoint(Waypoint waypoint, bool on) async {
    SegmentModel segment = context.read();
    segment.makeControlAtWaypoint(waypoint, on);
    KindsModel kinds = context.read();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      // KindsModel wants to know how many controls there are.
      kinds.updateStatistics(segment.statistics());
    });
  }

  Waypoint? previousControl(List<Waypoint> waypoints, int index) {
    if (index <= 0) {
      return null;
    }
    int startControlIndex = waypoints.indexWhere((waypoint) {
      return waypoint.origin == Kind.controls;
    });

    for (int i = index - 1; i >= 0; --i) {
      Waypoint candidate = waypoints[i];
      if (candidate.origin == Kind.controls) {
        if (candidate.hasCustomTime || i == startControlIndex) {
          return candidate;
        }
      }
    }
    return null;
  }

  Waypoint? nextControl(List<Waypoint> waypoints, int index) {
    if (index >= (waypoints.length - 1)) {
      return null;
    }

    int endControlIndex = waypoints.lastIndexWhere((waypoint) {
      return waypoint.origin == Kind.controls;
    });

    for (int i = index + 1; i < waypoints.length; ++i) {
      Waypoint candidate = waypoints[i];
      if (candidate.origin == Kind.controls) {
        if (candidate.hasCustomTime || i == endControlIndex) {
          return candidate;
        }
      }
    }
    return null;
  }

  Widget buildData(List<Waypoint> waypoints, Parameters parameters) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No waypoints"));
    }
    debugPrint("build table with ${waypoints.length} waypoints");

    // Find indices of first and last control
    int firstControlIndex = -1;
    int lastControlIndex = -1;
    for (int i = 0; i < waypoints.length; i++) {
      if (waypoints[i].origin == Kind.controls) {
        if (firstControlIndex == -1) {
          firstControlIndex = i;
        }
        lastControlIndex = i;
      }
    }
    SpeedMode speedMode = parseSpeedMode(parameters.speed);
    DateTime? currentDate;
    List<DataRow> rows = [];
    for (int entryIndex = 0; entryIndex < waypoints.length; entryIndex++) {
      final entry = MapEntry(entryIndex, waypoints[entryIndex]);
      final index = entry.key;
      final w = entry.value;

      // Check if we crossed midnight
      final waypointDate = parseDateTime(w.info!.time);
      bool crossedMidnight = false;
      if (currentDate != null) {
        crossedMidnight =
            waypointDate.year != currentDate.year ||
            waypointDate.month != currentDate.month ||
            waypointDate.day != currentDate.day;
      }
      if (crossedMidnight) {
        Widget line = Padding(
          padding: const EdgeInsets.symmetric(vertical: 0.0),
          child: Container(
            height: 1,
            color: const Color.fromARGB(255, 100, 100, 100),
            child: const SizedBox.expand(),
          ),
        );
        rows.add(
          DataRow(
            cells: <DataCell>[
              DataCell(line),
              DataCell(
                Center(
                  child: Text(
                    DateFormat('EEEE').format(waypointDate),
                    style: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
                  ),
                ),
              ),
              DataCell(line),
              if (widget.editControls) DataCell(line),
            ],
          ),
        );
      }
      currentDate = waypointDate;

      final formattedDistance = _formatDistance(w.info!.distance);
      final description = joinNonEmpty([w.name, w.description]);
      final isGpxWaypoint = w.origin == Kind.gpxWaypoints;
      final isControl = w.origin == Kind.controls;
      final isEditableControl =
          widget.editControls &&
          isControl &&
          speedMode == SpeedMode.kmh &&
          index != firstControlIndex &&
          index != lastControlIndex;
      final showCheckbox = isControl || isGpxWaypoint;
      final checkBoxValue = isControl;

      Widget timeWidget =
          isEditableControl
              ? ControlEditTimeButton(
                parameters: parameters,
                previousControl: previousControl(waypoints, index),
                nextControl: nextControl(waypoints, index),
                currentControl: w,
                onTimeChanged: (DateTime newDateTime) async {
                  SegmentModel segment = context.read();
                  segment.setControlTime(w, newDateTime);

                  if (!mounted) {
                    return;
                  }
                  FutureRenderer renderer = context.read();
                  renderer.reset();
                },
              )
              : makeTimeLabel(w, FontWeight.normal);

      DataRow row = DataRow(
        cells: <DataCell>[
          DataCell(Text(formattedDistance)),
          DataCell(timeWidget),
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
                          FutureRenderer renderer = context.read();
                          renderer.reset();
                        }
                      });
                    },
                  )
                  : const SizedBox.shrink(),
            ),
        ],
      );
      rows.add(row);
    }

    return DataTable(
      columnSpacing: 15.0,
      horizontalMargin: 10.0,
      headingRowHeight: 32.0,
      dataRowMinHeight: 30.0, // this does not seem to change anything.
      dataRowMaxHeight: 32.0,
      border: const TableBorder(
        verticalInside: BorderSide(width: 0.3, style: BorderStyle.solid),
      ),
      columns: <DataColumn>[
        const DataColumn(
          label: Text('KM', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        const DataColumn(
          label: Text('CUTOFF', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        const DataColumn(label: Text(""), numeric: false),
        if (widget.editControls)
          const DataColumn(
            label: Text(
              'Control\nPoint',
              textAlign: TextAlign.center,
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12.0),
            ),
            numeric: true,
          ),
      ],
      rows: rows,
    );
  }

  @override
  Widget build(BuildContext context) {
    SegmentModel model = context.watch<SegmentModel>();
    debugPrint("build table for segment ${model.segment.id()}");
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: buildData(widget.waypoints, model.parameters()),
    );
  }
}

class GPXTable extends StatelessWidget {
  final Kinds kinds;

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
    SegmentModel model = context.watch<SegmentModel>();
    context.watch<SegmentModel>();
    var waypoints = model.someWaypoints(kinds);
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: buildData(waypoints),
    );
  }
}
