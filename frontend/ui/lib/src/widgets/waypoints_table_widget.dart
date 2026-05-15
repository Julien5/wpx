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

class ControlEditTimeButton extends StatelessWidget {
  final Waypoint? previousControl;
  final Waypoint? nextControl;
  final String currentTimeIso;
  final Widget label;
  final Function(DateTime) onTimeChanged;
  const ControlEditTimeButton({
    super.key,
    required this.previousControl,
    this.nextControl,
    required this.currentTimeIso,
    required this.label,
    required this.onTimeChanged,
  });

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: () {
        openControlTimeDialog(
          context: context,
          previousControl: previousControl,
          nextControl: nextControl,
          currentTimeIso: currentTimeIso,
          onTimeChanged: onTimeChanged,
        );
      },
      style: ElevatedButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 4.0),
        minimumSize: const Size(0, 0),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      ),
      child: label,
    );
  }
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

  Waypoint? previousControl(List<Waypoint> waypoints, int index) {
    if (index <= 0) {
      return null;
    }
    for (int i = index - 1; i >= 0; --i) {
      if (waypoints[i].origin == Kind.controls) {
        return waypoints[i];
      }
    }
    return null;
  }

  Waypoint? nextControl(List<Waypoint> waypoints, int index) {
    if (index >= (waypoints.length - 1)) {
      return null;
    }
    for (int i = index + 1; i < waypoints.length; ++i) {
      if (waypoints[i].origin == Kind.controls) {
        return waypoints[i];
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

    return DataTable(
      columnSpacing: 15.0,
      horizontalMargin: 10.0,
      headingRowHeight: 32.0,
      dataRowMinHeight: 28.0,
      dataRowMaxHeight: 40.0,
      border: const TableBorder(
        verticalInside: BorderSide(width: 0.3, style: BorderStyle.solid),
      ),
      columns: <DataColumn>[
        const DataColumn(
          label: Text('KM', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        const DataColumn(
          label: Text('TIME', style: TextStyle(fontWeight: FontWeight.bold)),
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
      rows:
          waypoints.asMap().entries.map((entry) {
            final index = entry.key;
            final w = entry.value;
            final formattedDistance = _formatDistance(w.info!.distance);
            Widget labelText = Text(
              _formatTime(w.info!.time),
              style: const TextStyle(fontSize: 12),
            );
            if (w.hasCustomTime) {
              labelText = Text(
                _formatTime(w.info!.time),
                style: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
              );
            }
            final description = joinNonEmpty([w.name, w.description]);
            final isGpxWaypoint = w.origin == Kind.gpxWaypoints;
            final isControl = w.origin == Kind.controls;
            final isEditableControl =
                isControl &&
                speedMode == SpeedMode.constant &&
                index != firstControlIndex &&
                index != lastControlIndex;
            final showCheckbox = isControl || isGpxWaypoint;
            final checkBoxValue = isControl;

            Widget timeWidget =
                isEditableControl
                    ? ControlEditTimeButton(
                      previousControl: previousControl(waypoints, index),
                      nextControl: nextControl(waypoints, index),
                      currentTimeIso: w.info!.time,
                      label: labelText,
                      onTimeChanged: (DateTime newDateTime) {
                        SegmentModel segment = Provider.of(
                          context,
                          listen: false,
                        );
                        segment.setControlTime(w, newDateTime);
                        FutureRenderer renderer = Provider.of(
                          context,
                          listen: false,
                        );
                        renderer.reset();
                      },
                    )
                    : labelText;

            return DataRow(
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
                                FutureRenderer renderer = Provider.of(
                                  context,
                                  listen: false,
                                );
                                renderer.reset();
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
    ParameterModel parameters = Provider.of<ParameterModel>(context);
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: buildData(widget.waypoints, parameters.parameters()),
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
